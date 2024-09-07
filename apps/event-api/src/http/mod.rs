use std::sync::Arc;

use anyhow::Context;
use axum::extract::{MatchedPath, Request};
use axum::response::Response;
use axum::Router;
use hyper::StatusCode;
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::global::Global;

pub mod socket;
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

fn routes(global: &Arc<Global>) -> Router {
	Router::new()
		.nest("/v3", v3::routes())
		.with_state(Arc::clone(global))
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
							span.record("response.status_code", res.status().as_u16());
						}),
				)
				.layer(SetRequestIdLayer::x_request_id(TraceRequestId))
				.layer(PropagateRequestIdLayer::x_request_id()),
		)
}

async fn not_found() -> (StatusCode, &'static str) {
	(StatusCode::NOT_FOUND, "not found")
}

#[tracing::instrument(name = "api", level = "info", skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let config = global.config();

	let mut server = scuffle_foundations::http::server::Server::builder()
		.bind(config.api.bind)
		.build(routes(&global))
		.context("failed to build HTTP server")?;

	server.start().await.context("failed to start HTTP server")?;

	server.wait().await.context("HTTP server failed")?;

	Ok(())
}
