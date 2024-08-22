use std::sync::Arc;

use anyhow::Context;
use http::HeaderValue;
use quinn::crypto::rustls::QuicServerConfig;
use scuffle_foundations::http::server::axum::extract::{MatchedPath, Request};
use scuffle_foundations::http::server::axum::response::Response;
use scuffle_foundations::http::server::axum::Router;
use scuffle_foundations::http::server::stream::{Body, IncomingConnection, IntoResponse, MakeService, ServiceHandler};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::Span;

use crate::global::Global;

mod cdn;

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
		.nest("/", cdn::routes())
		.with_state(Arc::clone(global))
		.layer(
			ServiceBuilder::new()
				.layer(SetResponseHeaderLayer::overriding(
					http::header::SERVER,
					server_name.parse::<HeaderValue>().unwrap(),
				))
				.option_layer(if global.config.cdn.http3 && global.config.cdn.tls.is_some() {
					Some(SetResponseHeaderLayer::overriding(
						http::header::ALT_SVC,
						format!("h3=\":{}\"; ma=2592000", global.config.cdn.secure_bind.port())
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
							span.record("response.status_code", &res.status().as_u16());
						}),
				)
				.layer(SetRequestIdLayer::x_request_id(TraceRequestId))
				.layer(PropagateRequestIdLayer::x_request_id()),
		)
		.layer(CorsLayer::permissive())
}

#[derive(Clone, Copy, Debug)]
enum AnyService<A: ServiceHandler, B: ServiceHandler> {
	Left(A),
	Right(B),
}

impl<A: ServiceHandler, B: ServiceHandler> ServiceHandler for AnyService<A, B> {
	fn on_error(&self, err: scuffle_foundations::http::server::Error) -> impl std::future::Future<Output = ()> + Send {
		async move {
			match self {
				Self::Left(a) => a.on_error(err).await,
				Self::Right(b) => b.on_error(err).await,
			}
		}
	}

	fn on_hijack(&self) -> impl std::future::Future<Output = ()> + Send {
		async move {
			match self {
				Self::Left(a) => a.on_hijack().await,
				Self::Right(b) => b.on_hijack().await,
			}
		}
	}

	fn on_ready(&self) -> impl std::future::Future<Output = ()> + Send {
		async move {
			match self {
				Self::Left(a) => a.on_ready().await,
				Self::Right(b) => b.on_ready().await,
			}
		}
	}

	fn on_request(
		&self,
		req: Request,
	) -> impl std::future::Future<Output = impl scuffle_foundations::http::server::stream::IntoResponse> + Send {
		async move {
			match self {
				Self::Left(a) => a.on_request(req).await.into_response(),
				Self::Right(b) => b.on_request(req).await.into_response(),
			}
		}
	}

	fn on_close(&self) -> impl std::future::Future<Output = ()> + Send {
		async move {
			match self {
				Self::Left(a) => a.on_close().await,
				Self::Right(b) => b.on_close().await,
			}
		}
	}
}

#[derive(Clone)]
struct CustomMakeService {
	redirect_uncrypted: Option<u16>,
	server_name: Arc<str>,
	routes: Router,
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
		let Some(host) = req.headers().get(http::header::HOST).and_then(|h| h.to_str().ok()) else {
			return std::future::ready(http::StatusCode::BAD_REQUEST.into_response());
		};

		let mut builder = http::Uri::builder().scheme("https");

		if let Some((host, _)) = host.split_once(':') {
			builder = builder.authority(format!("{host}:{}", self.port));
		} else {
			builder = builder.authority(host);
		}

		if let Some(path_and_query) = req.uri().path_and_query() {
			builder = builder.path_and_query(path_and_query.clone());
		}

		let Ok(uri) = builder.build() else {
			return std::future::ready(http::StatusCode::BAD_REQUEST.into_response());
		};

		let Ok(response) = http::Response::builder()
			.status(http::StatusCode::PERMANENT_REDIRECT)
			.header(http::header::LOCATION, uri.to_string())
			.header(
				http::header::SERVER,
				self.server_name.as_ref().parse::<HeaderValue>().unwrap(),
			)
			.body(Body::empty())
		else {
			return std::future::ready(http::StatusCode::BAD_REQUEST.into_response());
		};

		std::future::ready(response)
	}
}

impl MakeService for CustomMakeService {
	fn make_service(
		&self,
		incoming: &impl IncomingConnection,
	) -> impl std::future::Future<Output = Option<impl ServiceHandler>> + Send {
		let service = if let (Some(port), false) = (self.redirect_uncrypted, incoming.is_encrypted()) {
			AnyService::Right(RedirectUncrypted {
				port,
				server_name: self.server_name.clone(),
			})
		} else {
			AnyService::Left(self.routes.clone())
		};

		std::future::ready(Some(service))
	}
}

#[tracing::instrument(name = "cdn", level = "info", skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let mut builder = scuffle_foundations::http::server::Server::builder()
		.with_workers(global.config.cdn.workers)
		.with_http({
			let mut http = hyper_util::server::conn::auto::Builder::new(Default::default());

			http.http1().max_buf_size(8_192);

			http.http2().max_send_buf_size(8_192);

			http
		});

	if let Some(tls) = global.config.cdn.tls.as_ref() {
		let cert = tokio::fs::read(&tls.cert).await.context("failed to read cert")?;
		let key = tokio::fs::read(&tls.key).await.context("failed to read key")?;

		let certs = rustls_pemfile::certs(&mut std::io::Cursor::new(cert))
			.collect::<std::result::Result<Vec<_>, _>>()
			.context("invalid cert")?;
		let key = rustls_pemfile::pkcs8_private_keys(&mut std::io::Cursor::new(key))
			.next()
			.context("missing key")?
			.context("invalid key")?;

		let mut tls_config = rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
			.with_no_client_auth()
			.with_single_cert(certs, key.into())
			.context("failed to build tls config")?;

		tls_config.max_early_data_size = u32::MAX;

		tls_config.alpn_protocols = tls.alpn_protocols.iter().map(|p| p.clone().into_bytes()).collect();

		if global.config.cdn.http3 {
			let server_config = quinn::ServerConfig::with_crypto(Arc::new(
				QuicServerConfig::try_from(tls_config.clone()).context("failed to build quic config")?,
			));
			builder = builder.with_http3(h3::server::builder(), server_config);
		}

		builder = builder
			.bind(global.config.cdn.secure_bind)
			.with_tls(tls_config)
			.with_insecure(global.config.cdn.bind);
	} else {
		builder = builder.bind(global.config.cdn.bind);
	}

	let server_name = global.config.cdn.server_name.clone().into();

	let mut server = builder
		.build(CustomMakeService {
			routes: routes(&global, &server_name),
			server_name,
			redirect_uncrypted: if !global.config.cdn.allow_insecure && global.config.cdn.tls.is_some() {
				Some(global.config.cdn.secure_bind.port())
			} else {
				None
			},
		})
		.context("failed to build HTTP server")?;

	server.start().await.context("failed to start HTTP server")?;

	server.wait().await.context("HTTP server failed")?;

	Ok(())
}
