use std::sync::Arc;

use anyhow::Context as _;
use axum::extract::{MatchedPath, Request};
use axum::response::Response;
use axum::Router;
use scuffle_foundations::context::Context;
use scuffle_foundations::telementry::opentelemetry::OpenTelemetrySpanExt;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::Span;

use self::error::ApiError;
use self::middleware::auth::AuthMiddleware;
use self::middleware::cookies::CookieMiddleware;
use crate::global::Global;

pub mod error;
pub mod extract;
pub mod middleware;
pub mod v3;

#[derive(Clone)]
struct TraceRequestId;

impl MakeRequestId for TraceRequestId {
	fn make_request_id<B>(&mut self, _: &hyper::Request<B>) -> Option<RequestId> {
		tracing::Span::current()
			.trace_id()
			.and_then(|id| id.to_string().parse().ok())
			.map(RequestId::new)
	}
}

fn routes(global: Arc<Global>) -> Router {
	Router::new()
		.nest("/v3", v3::routes())
		.with_state(global.clone())
		.fallback(not_found)
		.layer(
			ServiceBuilder::new()
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
						.on_response(|res: &Response, _, span: &Span| {
							span.record("response.status_code", &res.status().as_u16());
						}),
				)
				.layer(SetRequestIdLayer::x_request_id(TraceRequestId))
				.layer(PropagateRequestIdLayer::x_request_id())
				.layer(CookieMiddleware)
				.layer(AuthMiddleware::new(global)),
		)
}

#[tracing::instrument]
pub async fn not_found() -> ApiError {
	ApiError::NOT_FOUND
}

#[tracing::instrument(skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let config = global.config();

	let tcp_listener = tokio::net::TcpListener::bind(config.api.bind).await?;

	tracing::info!("listening on {}", tcp_listener.local_addr()?);

	axum::serve(tcp_listener, routes(global))
		.with_graceful_shutdown(Context::global().into_done())
		.await
		.context("http server failed")?;

	Ok(())
}
