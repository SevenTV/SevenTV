use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use axum::extract::{MatchedPath, Request, State};
use axum::http::HeaderName;
use axum::response::Response;
use axum::routing::get;
use axum::{Extension, Router};
use error::ApiErrorCode;
use hyper::Method;
use middleware::ip::IpMiddleware;
use middleware::session::{Session, SessionMiddleware};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use tower::ServiceBuilder;
use tower_http::cors::{AllowCredentials, AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders, MaxAge};
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::Span;

use self::error::ApiError;
use self::middleware::cookies::CookieMiddleware;
use crate::global::Global;

pub mod egvault;
pub mod error;
pub mod extract;
pub mod guards;
pub mod internal;
pub mod middleware;
pub mod v3;
pub mod v4;
pub mod validators;

const ALLOWED_CORS_HEADERS: [&str; 8] = [
	"content-type",
	"content-length",
	"accept-encoding",
	"authorization",
	"cookie",
	"x-emote-data",
	"x-seventv-platform",
	"x-seventv-version",
];

fn cors_layer(global: &Arc<Global>) -> CorsLayer {
	let website_origin = global.config.api.website_origin.clone();
	let api_origin = global.config.api.api_origin.clone();
	let allow_credentials = AllowCredentials::predicate(move |origin, _| {
		origin
			.to_str()
			.ok()
			.and_then(|o| url::Url::parse(o).ok())
			.map(|o| o.origin() == website_origin.origin() || o.origin() == api_origin.origin())
			.unwrap_or_default()
	});

	CorsLayer::new()
		.allow_origin(AllowOrigin::mirror_request())
		.allow_credentials(allow_credentials)
		.allow_methods(AllowMethods::list([
			Method::GET,
			Method::POST,
			Method::PUT,
			Method::PATCH,
			Method::DELETE,
		]))
		.allow_headers(AllowHeaders::list(
			ALLOWED_CORS_HEADERS.into_iter().map(HeaderName::from_static),
		))
		.expose_headers(ExposeHeaders::list([HeaderName::from_static("x-access-token")]))
		.max_age(MaxAge::exact(Duration::from_secs(7200)))
}

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
		.route("/", get(root))
		.nest("/internal", internal::routes())
		.nest("/v3", v3::routes(&global))
		.nest("/v4", v4::routes(&global))
		.nest("/egvault/v1", egvault::routes())
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
							span.record("response.status_code", res.status().as_u16());
						}),
				)
				.layer(SetRequestIdLayer::x_request_id(TraceRequestId))
				.layer(PropagateRequestIdLayer::x_request_id())
				.layer(IpMiddleware::new(global.clone()))
				.layer(CookieMiddleware)
				.layer(SessionMiddleware::new(global.clone()))
				.layer(cors_layer(&global)),
		)
}

#[derive(serde::Serialize)]
struct RootResp {
	message: &'static str,
	version: &'static str,
	ip: std::net::IpAddr,
	country: Option<String>,
}

#[tracing::instrument(skip_all)]
async fn root(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
) -> impl axum::response::IntoResponse {
	let resp = RootResp {
		message: "Welcome to the 7TV API!",
		version: env!("CARGO_PKG_VERSION"),
		ip: session.ip(),
		country: global
			.geoip()
			.and_then(|geoip| geoip.lookup(session.ip()))
			.and_then(|l| l.iso_code)
			.map(Into::into),
	};

	axum::Json(resp)
}

#[tracing::instrument]
pub async fn not_found() -> ApiError {
	ApiError::not_found(ApiErrorCode::BadRequest, "route not found")
}

#[tracing::instrument(name = "API", skip(global))]
pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	let mut server = scuffle_foundations::http::server::Server::builder()
		.bind(global.config.api.bind)
		.with_workers(global.config.api.workers)
		.build(routes(global))
		.context("Failed to build HTTP server")?;

	server.start().await.context("Failed to start HTTP server")?;

	server.wait().await.context("HTTP server failed")?;

	Ok(())
}
