use std::sync::Arc;

use ::http::HeaderValue;
use anyhow::Context;
use axum::extract::{MatchedPath, Request};
use axum::response::Response;
use axum::routing::any;
use axum::Router;
use scuffle_context::ContextFutExt;
use scuffle_http::backend::HttpServer;
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

mod socket;
mod v3;

fn routes(global: &Arc<Global>, server_name: &Arc<str>) -> Router {
	Router::new()
		.route("/v3", any(v3::handle))
		.route("/v3*any", any(v3::handle))
		.with_state(Arc::clone(global))
		.layer(
			ServiceBuilder::new()
				.layer(SetResponseHeaderLayer::overriding(
					::http::header::SERVER,
					server_name.parse::<HeaderValue>().unwrap(),
				))
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
				.layer(IpMiddleware::new(global.config.event_api.incoming_request.clone())),
		)
		.layer(CorsLayer::permissive())
}

pub async fn run(global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
	let server_name: Arc<str> = global.config.event_api.server_name.clone().into();

	let routes = routes(&global, &server_name);

	let builder = scuffle_http::backend::tcp::TcpServerConfig::builder()
		.with_http_builder_fn(|| {
			let mut http = hyper_util::server::conn::auto::Builder::new(Default::default());

			http.http1().max_buf_size(8_192);

			http.http2().max_send_buf_size(8_192);

			http
		})
		.with_server_name(server_name.clone())
		.with_bind(global.config.event_api.bind);

	let server = if let Some(tls) = global.config.event_api.tls.as_ref() {
		let cert = tokio::fs::read(&tls.cert).await.context("failed to read cert")?;
		let key = tokio::fs::read(&tls.key).await.context("failed to read key")?;

		builder
			.with_tls_from_pem(cert, key)
			.context("failed to build tls config")?
			.build()
	} else {
		builder.build()
	}
	.into_server();

	let limiter = RateLimiter::new(&global.config.event_api.rate_limit);

	server
		.start(
			MonitorAcceptor::new(
				scuffle_http::svc::axum_service(routes),
				if global.config.event_api.tls.is_some() {
					SocketKind::TlsTcp
				} else {
					SocketKind::Tcp
				},
				limiter.clone(),
			),
			global.config.event_api.workers,
		)
		.await
		.context("start tcp server")?;

	server
		.wait()
		.with_context(&ctx)
		.await
		.transpose()
		.context("HTTP server failed")?;

	server.shutdown().await.context("failed to shutdown HTTP server")?;

	Ok(())
}
