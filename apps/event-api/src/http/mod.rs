use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Instant;

use ::http::HeaderValue;
use anyhow::Context;
use quinn::crypto::rustls::QuicServerConfig;
use scuffle_foundations::http::server::axum::body::HttpBody;
use scuffle_foundations::http::server::axum::extract::{MatchedPath, Request};
use scuffle_foundations::http::server::axum::response::Response;
use scuffle_foundations::http::server::axum::Router;
use scuffle_foundations::http::server::stream::{Body, IncomingConnection, IntoResponse, MakeService, ServiceHandler};
use scuffle_foundations::telemetry::metrics::metrics;
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::http::ip::IpMiddleware;
use shared::http::ratelimit::{RateLimitDropGuard, RateLimiter};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::Span;

use self::http::{ActionKind, ConnectionDropGuard, SocketKind};
use crate::global::Global;

mod socket;
mod v3;

#[derive(Clone)]
struct TraceRequestId;

impl MakeRequestId for TraceRequestId {
	fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
		tracing::Span::current()
			.trace_id()
			.and_then(|id| id.to_string().parse().ok())
			.map(RequestId::new)
	}
}

fn routes(global: &Arc<Global>, server_name: &Arc<str>) -> Router {
	Router::new()
		.nest("/v3", v3::routes())
		.with_state(Arc::clone(global))
		.layer(
			ServiceBuilder::new()
				.layer(SetResponseHeaderLayer::overriding(
					::http::header::SERVER,
					server_name.parse::<HeaderValue>().unwrap(),
				))
				.option_layer(if global.config.event_api.http3 && global.config.event_api.tls.is_some() {
					Some(SetResponseHeaderLayer::overriding(
						::http::header::ALT_SVC,
						format!("h3=\":{}\"; ma=2592000", global.config.event_api.secure_bind.port())
							.parse::<HeaderValue>()
							.unwrap(),
					))
				} else {
					None
				})
				.layer(
					TraceLayer::new_for_http()
						.make_span_with(|req: &Request| {
							let matched_path = req.extensions().get::<MatchedPath>().map(MatchedPath::as_str);

							let span = tracing::info_span!(
								"request",
								"request.method" = %req.method(),
								"request.uri" = %req.uri(),
								"request.matched_path" = %matched_path.unwrap_or("<not found>"),
								"response.status_code" = tracing::field::Empty,
							);

							span.make_root();

							span
						})
						.on_failure(DefaultOnFailure::new().level(tracing::Level::DEBUG))
						.on_response(|res: &Response, _, span: &Span| {
							span.record("response.status_code", res.status().as_u16());
						}),
				)
				.layer(IpMiddleware::new(global.config.event_api.incoming_request.clone()))
				.layer(SetRequestIdLayer::x_request_id(TraceRequestId))
				.layer(PropagateRequestIdLayer::x_request_id()),
		)
		.layer(CorsLayer::permissive())
}

#[derive(Clone)]
struct AnyService<A: ServiceHandler, B: ServiceHandler> {
	kind: SocketKind,
	started_at: Instant,
	request_count: Arc<AtomicUsize>,
	_guard: Arc<(ConnectionDropGuard, RateLimitDropGuard)>,
	inner: AnyServiceInner<A, B>,
}

impl<A: ServiceHandler, B: ServiceHandler> AnyService<A, B> {
	fn new_a(kind: SocketKind, svc: A, limiter: RateLimitDropGuard) -> Self {
		Self::new(kind, AnyServiceInner::Left(svc), limiter)
	}

	fn new_b(kind: SocketKind, svc: B, limiter: RateLimitDropGuard) -> Self {
		Self::new(kind, AnyServiceInner::Right(svc), limiter)
	}

	fn new(kind: SocketKind, svc: AnyServiceInner<A, B>, limiter: RateLimitDropGuard) -> Self {
		Self {
			kind,
			request_count: Arc::new(AtomicUsize::new(0)),
			started_at: Instant::now(),
			_guard: Arc::new((ConnectionDropGuard::new(kind), limiter)),
			inner: svc,
		}
	}
}

#[derive(Clone, Copy, Debug)]
enum AnyServiceInner<A: ServiceHandler, B: ServiceHandler> {
	Left(A),
	Right(B),
}

#[metrics]
mod http {
	use scuffle_foundations::http::server::stream;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::counter::Counter;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::gauge::Gauge;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::histogram::Histogram;
	use scuffle_foundations::telemetry::metrics::HistogramBuilder;
	use serde::{Deserialize, Serialize};

	pub struct ConnectionDropGuard(SocketKind);

	impl Drop for ConnectionDropGuard {
		fn drop(&mut self) {
			connections(self.0).dec();
		}
	}

	impl ConnectionDropGuard {
		pub fn new(socket: SocketKind) -> Self {
			connections(socket).inc();
			Self(socket)
		}
	}

	pub fn connections(socket: SocketKind) -> Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub enum ActionKind {
		Error,
		Hijack,
		Ready,
		Request,
		Close,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	pub enum SocketKind {
		Tcp,
		TlsTcp,
		Quic,
	}

	impl From<stream::SocketKind> for SocketKind {
		fn from(value: stream::SocketKind) -> Self {
			match value {
				stream::SocketKind::Tcp => Self::Tcp,
				stream::SocketKind::TlsTcp => Self::TlsTcp,
				stream::SocketKind::Quic => Self::Quic,
			}
		}
	}

	pub fn actions(socket: SocketKind, action: ActionKind) -> Counter;

	pub fn status_code(socket: SocketKind, status: String) -> Counter;

	#[builder = HistogramBuilder::default()]
	pub fn socket_request_count(socket: SocketKind) -> Histogram;

	#[builder = HistogramBuilder::default()]
	pub fn socket_duration(socket: SocketKind) -> Histogram;

	#[builder = HistogramBuilder::default()]
	pub fn request_duration(socket: SocketKind) -> Histogram;

	pub fn bytes_sent(socket: SocketKind) -> Counter;
}

impl<A: ServiceHandler, B: ServiceHandler> ServiceHandler for AnyService<A, B> {
	fn on_error(&self, err: scuffle_foundations::http::server::Error) -> impl std::future::Future<Output = ()> + Send {
		http::actions(self.kind, ActionKind::Error).inc();

		tracing::debug!("error while handling request: {:#}", err);

		async move {
			match &self.inner {
				AnyServiceInner::Left(a) => a.on_error(err).await,
				AnyServiceInner::Right(b) => b.on_error(err).await,
			}
		}
	}

	fn on_hijack(&self) -> impl std::future::Future<Output = ()> + Send {
		http::actions(self.kind, ActionKind::Hijack).inc();

		async move {
			match &self.inner {
				AnyServiceInner::Left(a) => a.on_hijack().await,
				AnyServiceInner::Right(b) => b.on_hijack().await,
			}
		}
	}

	fn on_ready(&self) -> impl std::future::Future<Output = ()> + Send {
		http::actions(self.kind, ActionKind::Ready).inc();

		async move {
			match &self.inner {
				AnyServiceInner::Left(a) => a.on_ready().await,
				AnyServiceInner::Right(b) => b.on_ready().await,
			}
		}
	}

	fn on_request(
		&self,
		req: Request,
	) -> impl std::future::Future<Output = impl scuffle_foundations::http::server::stream::IntoResponse> + Send {
		http::actions(self.kind, ActionKind::Request).inc();

		self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

		let start = Instant::now();

		async move {
			let request = match &self.inner {
				AnyServiceInner::Left(a) => a.on_request(req).await.into_response(),
				AnyServiceInner::Right(b) => b.on_request(req).await.into_response(),
			};

			let size = request.body().size_hint();

			http::bytes_sent(self.kind).inc_by(size.exact().unwrap_or(size.lower()));
			http::status_code(self.kind, request.status().as_u16().to_string()).inc();
			http::request_duration(self.kind).observe(start.elapsed().as_secs_f64());

			request
		}
	}

	fn on_close(&self) -> impl std::future::Future<Output = ()> + Send {
		http::actions(self.kind, ActionKind::Close).inc();

		http::socket_duration(self.kind).observe(self.started_at.elapsed().as_secs_f64());
		http::socket_request_count(self.kind).observe(self.request_count.load(std::sync::atomic::Ordering::Relaxed) as f64);

		async move {
			match &self.inner {
				AnyServiceInner::Left(a) => a.on_close().await,
				AnyServiceInner::Right(b) => b.on_close().await,
			}
		}
	}
}

#[derive(Clone)]
struct CustomMakeService {
	redirect_uncrypted: Option<u16>,
	server_name: Arc<str>,
	routes: Router,
	limiter: Arc<RateLimiter>,
}

#[derive(Clone, Debug)]
struct RedirectUncrypted {
	port: u16,
	server_name: Arc<str>,
}

impl ServiceHandler for RedirectUncrypted {
	fn on_request(
		&self,
		req: Request,
	) -> impl std::future::Future<Output = impl scuffle_foundations::http::server::stream::IntoResponse> + Send {
		let Some(host) = req.headers().get(::http::header::HOST).and_then(|h| h.to_str().ok()) else {
			return std::future::ready(::http::StatusCode::BAD_REQUEST.into_response());
		};

		let mut builder = ::http::Uri::builder().scheme("https");

		{
			let Some(uri) = format!("https://{host}").parse::<::http::Uri>().ok() else {
				return std::future::ready(::http::StatusCode::BAD_REQUEST.into_response());
			};

			let Some(host) = uri.host() else {
				return std::future::ready(::http::StatusCode::BAD_REQUEST.into_response());
			};

			if self.port != 443 {
				builder = builder.authority(format!("{host}:{}", self.port));
			} else {
				builder = builder.authority(host);
			}
		}

		if let Some(path_and_query) = req.uri().path_and_query() {
			builder = builder.path_and_query(path_and_query.clone());
		}

		let Ok(uri) = builder.build() else {
			return std::future::ready(::http::StatusCode::BAD_REQUEST.into_response());
		};

		let Ok(response) = ::http::Response::builder()
			.status(::http::StatusCode::PERMANENT_REDIRECT)
			.header(::http::header::LOCATION, uri.to_string())
			.header(
				::http::header::SERVER,
				self.server_name.as_ref().parse::<HeaderValue>().unwrap(),
			)
			.body(Body::empty())
		else {
			return std::future::ready(::http::StatusCode::BAD_REQUEST.into_response());
		};

		std::future::ready(response)
	}
}

impl MakeService for CustomMakeService {
	fn make_service(
		&self,
		incoming: &impl IncomingConnection,
	) -> impl std::future::Future<Output = Option<impl ServiceHandler>> + Send {
		let Some(ticket) = self.limiter.acquire(incoming.remote_addr().ip()) else {
			return std::future::ready(None);
		};

		let service = if let (Some(port), false) = (self.redirect_uncrypted, incoming.is_encrypted()) {
			AnyService::new_b(
				incoming.socket_kind().into(),
				RedirectUncrypted {
					port,
					server_name: self.server_name.clone(),
				},
				ticket,
			)
		} else {
			AnyService::new_a(incoming.socket_kind().into(), self.routes.clone(), ticket)
		};

		std::future::ready(Some(service))
	}
}

#[tracing::instrument(name = "cdn", level = "info", skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let mut builder = scuffle_foundations::http::server::Server::builder()
		.with_workers(if global.config.event_api.workers == 0 {
			num_cpus::get()
		} else {
			global.config.event_api.workers
		})
		.with_http({
			let mut http = hyper_util::server::conn::auto::Builder::new(Default::default());

			http.http1().max_buf_size(8_192);

			http.http2().max_send_buf_size(8_192);

			http
		});

	if let Some(tls) = global.config.event_api.tls.as_ref() {
		let cert = tokio::fs::read(&tls.cert).await.context("failed to read cert")?;
		let key = tokio::fs::read(&tls.key).await.context("failed to read key")?;

		let certs = rustls_pemfile::certs(&mut std::io::Cursor::new(cert))
			.collect::<std::result::Result<Vec<_>, _>>()
			.context("invalid cert")?;
		let key = rustls_pemfile::pkcs8_private_keys(&mut std::io::Cursor::new(key))
			.next()
			.context("missing key")?
			.context("invalid key")?;

		let tls_config = rustls::ServerConfig::builder()
			.with_no_client_auth()
			.with_single_cert(certs, key.into())
			.context("failed to build tls config")?;

		if global.config.event_api.http3 {
			let mut tls_config = tls_config.clone();
			tls_config.max_early_data_size = u32::MAX;
			tls_config.alpn_protocols = vec![b"h3".to_vec()];

			let server_config = quinn::ServerConfig::with_crypto(Arc::new(
				QuicServerConfig::try_from(tls_config.clone()).context("failed to build quic config")?,
			));

			builder = builder.with_http3(h3::server::builder(), server_config);
		}

		builder = builder
			.bind(global.config.event_api.secure_bind)
			.with_tls(tls_config)
			.with_insecure(global.config.event_api.bind);
	} else {
		builder = builder.bind(global.config.event_api.bind);
	}

	let server_name = global.config.event_api.server_name.clone().into();

	let mut server = builder
		// Bug with keep-alive timeout, doesn't take in account streaming connections
		.with_keep_alive_timeout(None)
		.build(CustomMakeService {
			routes: routes(&global, &server_name),
			server_name,
			limiter: RateLimiter::new(&global.config.event_api.rate_limit),
			redirect_uncrypted: if !global.config.event_api.allow_insecure && global.config.event_api.tls.is_some() {
				Some(global.config.event_api.secure_bind.port())
			} else {
				None
			},
		})
		.context("failed to build HTTP server")?;

	server.start().await.context("failed to start HTTP server")?;

	server.wait().await.context("HTTP server failed")?;

	Ok(())
}
