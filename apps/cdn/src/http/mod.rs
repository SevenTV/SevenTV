use std::convert::Infallible;
use std::sync::Arc;

use ::http::{HeaderName, HeaderValue};
use anyhow::Context;
use axum::body::Body;
use axum::extract::{MatchedPath, Request};
use axum::response::{IntoResponse, Response};
use axum::Router;
use scuffle_http::backend::HttpServer;
use scuffle_http::body::IncomingBody;
use scuffle_http::svc::AxumService;
use shared::http::ip::IpMiddleware;
use shared::http::metrics::SocketKind;
use shared::http::ratelimit::RateLimiter;
use shared::http::MonitorAcceptor;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::Span;

use crate::global::Global;

mod cdn;

fn routes(global: &Arc<Global>, server_name: &Arc<str>) -> Router {
	Router::new()
		.nest("/", cdn::routes(global))
		.with_state(Arc::clone(global))
		.layer(
			ServiceBuilder::new()
				.layer(SetResponseHeaderLayer::overriding(
					::http::header::SERVER,
					server_name.parse::<HeaderValue>().unwrap(),
				))
				.layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
					HeaderName::from_static("x-7tv-cdn-node"),
					HeaderValue::from_str(&global.config.pod.node_name).unwrap(),
				))
				.layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
					HeaderName::from_static("x-7tv-cdn-pod"),
					HeaderValue::from_str(&global.config.pod.name).unwrap(),
				))
				.option_layer(if global.config.cdn.http3 && global.config.cdn.tls.is_some() {
					Some(SetResponseHeaderLayer::overriding(
						::http::header::ALT_SVC,
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

							span
						})
						.on_failure(DefaultOnFailure::new().level(tracing::Level::DEBUG))
						.on_response(|res: &Response, _, span: &Span| {
							span.record("response.status_code", res.status().as_u16());
						}),
				)
				.layer(IpMiddleware::new(global.config.cdn.incoming_request.clone())),
		)
		.layer(CorsLayer::permissive())
}

#[derive(Clone)]
struct InsecureHandler {
	server_name: Arc<str>,
	port: u16,
}

#[async_trait::async_trait]
impl scuffle_http::svc::ConnectionHandle for InsecureHandler {
	type Body = axum::body::Body;
	type BodyData = axum::body::Bytes;
	type BodyError = axum::Error;
	type Error = Infallible;

	async fn on_request(&self, req: ::http::Request<IncomingBody>) -> Result<::http::Response<Self::Body>, Self::Error> {
		Ok(handle_http_request(&req, &self.server_name, self.port))
	}
}

fn handle_http_request<B>(req: &Request<B>, server_name: &str, port: u16) -> axum::response::Response {
	let Some(host) = req.headers().get(::http::header::HOST).and_then(|h| h.to_str().ok()) else {
		return ::http::StatusCode::BAD_REQUEST.into_response();
	};

	let mut builder = ::http::Uri::builder().scheme("https");

	{
		let Some(uri) = format!("https://{host}").parse::<::http::Uri>().ok() else {
			return ::http::StatusCode::BAD_REQUEST.into_response();
		};

		let Some(host) = uri.host() else {
			return ::http::StatusCode::BAD_REQUEST.into_response();
		};

		if port != 443 {
			builder = builder.authority(format!("{host}:{}", port));
		} else {
			builder = builder.authority(host);
		}
	}

	if let Some(path_and_query) = req.uri().path_and_query() {
		builder = builder.path_and_query(path_and_query.clone());
	}

	let Ok(uri) = builder.build() else {
		return ::http::StatusCode::BAD_REQUEST.into_response();
	};

	let Ok(response) = ::http::Response::builder()
		.status(::http::StatusCode::PERMANENT_REDIRECT)
		.header(::http::header::LOCATION, uri.to_string())
		.header(::http::header::SERVER, server_name.parse::<HeaderValue>().unwrap())
		.body(Body::empty())
	else {
		return ::http::StatusCode::BAD_REQUEST.into_response();
	};

	response
}

#[derive(Clone)]
pub enum Either<A, B> {
	A(A),
	B(B),
}

#[async_trait::async_trait]
impl<A, B> scuffle_http::svc::ConnectionHandle for Either<A, B>
where
	A: scuffle_http::svc::ConnectionHandle,
	B: scuffle_http::svc::ConnectionHandle<
		Body = A::Body,
		BodyData = A::BodyData,
		BodyError = A::BodyError,
		Error = A::Error,
	>,
{
	type Body = A::Body;
	type BodyData = A::BodyData;
	type BodyError = A::BodyError;
	type Error = A::Error;

	async fn on_request(&self, req: ::http::Request<IncomingBody>) -> Result<::http::Response<Self::Body>, Self::Error> {
		match self {
			Either::A(a) => a.on_request(req).await,
			Either::B(b) => b.on_request(req).await,
		}
	}

	fn on_close(&self) {
		match self {
			Either::A(a) => a.on_close(),
			Either::B(b) => b.on_close(),
		}
	}

	fn on_error(&self, err: scuffle_http::Error) {
		match self {
			Either::A(a) => a.on_error(err),
			Either::B(b) => b.on_error(err),
		}
	}

	fn on_ready(&self) {
		match self {
			Either::A(a) => a.on_ready(),
			Either::B(b) => b.on_ready(),
		}
	}
}

pub async fn run(global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
	let tcp_server = scuffle_http::backend::tcp::TcpServerConfig::builder()
		.with_bind(global.config.cdn.bind)
		.build()
		.into_server();

	let (tls_server, quic_server) = if let Some(tls) = &global.config.cdn.tls {
		let cert = tokio::fs::read(&tls.cert).await.context("read cert")?;
		let key = tokio::fs::read(&tls.key).await.context("read key")?;

		let tls_server = scuffle_http::backend::tcp::TcpServerConfig::builder()
			.with_bind(global.config.cdn.secure_bind)
			.with_tls_from_pem(&cert, &key)
			.context("build tls server")?
			.build()
			.into_server();

		let quic_server = if global.config.cdn.http3 {
			Some(scuffle_http::backend::quic::quinn::QuinnServer::new(
				scuffle_http::backend::quic::quinn::QuinnServerConfig::builder()
					.with_bind(global.config.cdn.secure_bind)
					.with_tls_from_pem(&cert, &key)
					.context("build quic server")?
					.build(),
			))
		} else {
			None
		};

		(Some(tls_server), quic_server)
	} else {
		(None, None)
	};

	let workers = if global.config.cdn.workers == 0 {
		num_cpus::get()
	} else {
		global.config.cdn.workers
	};

	let server_name = global.config.cdn.server_name.clone().into();

	let handler = scuffle_http::svc::axum_service(routes(&global, &server_name));

	let insecure_handler: Either<InsecureHandler, AxumService<Router>> = if tls_server.is_some() {
		Either::A(InsecureHandler {
			port: global.config.cdn.secure_bind.port(),
			server_name: server_name.clone(),
		})
	} else {
		Either::B(handler.clone())
	};

	let limiter = RateLimiter::new(&global.config.cdn.rate_limit);

	tcp_server
		.start(
			MonitorAcceptor::new(insecure_handler, SocketKind::Tcp, limiter.clone()),
			workers,
		)
		.await
		.context("start tcp server")?;
	if let Some(tls_server) = &tls_server {
		tls_server
			.start(
				MonitorAcceptor::new(handler.clone(), SocketKind::TlsTcp, limiter.clone()),
				workers,
			)
			.await
			.context("start tls server")?;
	}
	if let Some(quic_server) = &quic_server {
		quic_server
			.start(MonitorAcceptor::new(handler.clone(), SocketKind::Quic, limiter), workers)
			.await
			.context("start quic server")?;
	}

	tokio::select! {
		r = tcp_server.wait() => r.context("tcp server")?,
		Some(r) = async { Some(tls_server.as_ref()?.wait().await) } => r.context("tls server")?,
		Some(r) = async { Some(quic_server.as_ref()?.wait().await) } => r.context("quic server")?,
		_ = ctx.done() => {}
	}

	tokio::try_join!(
		async { tcp_server.shutdown().await.context("tcp server") },
		async {
			if let Some(tls_server) = &tls_server {
				tls_server.shutdown().await.context("tls server")
			} else {
				Ok(())
			}
		},
		async {
			if let Some(quic_server) = &quic_server {
				quic_server.shutdown().await.context("quic server")
			} else {
				Ok(())
			}
		},
	)
	.context("shutdown")?;

	Ok(())
}
