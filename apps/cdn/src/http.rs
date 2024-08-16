use std::sync::Arc;

use anyhow::Context;
use reqwest::header::HeaderValue;
use scuffle_foundations::{
	http::server::axum::{
		self,
		extract::{MatchedPath, Path, Request, State},
		middleware::Next,
		response::Response,
		routing::any,
		Router,
	},
	telemetry::opentelemetry::OpenTelemetrySpanExt,
};
use tower::ServiceBuilder;
use tower_http::{
	cors::CorsLayer,
	request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
	trace::TraceLayer,
};
use tracing::Span;

use crate::global::Global;

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

fn routes(global: &Arc<Global>) -> Router {
	Router::new()
		.route("/", any(root))
		.route("/*key", any(cdn_route))
		.layer(axum::middleware::from_fn(move |req: Request, next: Next| async move {
			let mut res = next.run(req).await;
			res.headers_mut().insert("Server", HeaderValue::from_static("SevenTV"));
			res
		}))
		.with_state(Arc::clone(global))
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
				.layer(PropagateRequestIdLayer::x_request_id()),
		)
		.layer(CorsLayer::permissive())
}

async fn root() -> &'static str {
	"Welcome to the 7TV CDN!"
}

async fn cdn_route(State(_global): State<Arc<Global>>, Path(key): Path<String>) -> String {
	format!("Welcome to the 7TV CDN! {}", key)
}

#[tracing::instrument(name = "cdn", level = "info", skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let mut server = scuffle_foundations::http::server::Server::builder()
		.bind(global.config.cdn.bind)
		.build(routes(&global))
		.context("failed to build HTTP server")?;

	server.start().await.context("failed to start HTTP server")?;

	server.wait().await.context("HTTP server failed")?;

	Ok(())
}
