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
use middleware::session::{Session, SessionMiddleware};
use scuffle_context::ContextFutExt;
use scuffle_http::backend::HttpServer;
use shared::http::ip::IpMiddleware;
use shared::http::metrics::SocketKind;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowCredentials, AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders, MaxAge};
use tower_http::trace::{DefaultOnFailure, TraceLayer};
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

const ALLOWED_CORS_HEADERS: &[&str] = &[
	"content-type",
	"content-length",
	"accept-encoding",
	"authorization",
	"cookie",
	"x-emote-data",
	"x-seventv-platform",
	"x-seventv-version",
	"x-ignore-auth-failure",
];

fn cors_layer(global: &Arc<Global>) -> CorsLayer {
	let mut allowed_origins = global.config.api.cors_allowed_credential_origins.clone();
	allowed_origins.push(global.config.api.old_website_origin.clone());
	allowed_origins.push(global.config.api.website_origin.clone());
	allowed_origins.push(global.config.api.api_origin.clone());

	let allow_credentials = AllowCredentials::predicate(move |origin, _| {
		origin
			.to_str()
			.ok()
			.and_then(|o| url::Url::parse(o).ok())
			.map(|o| allowed_origins.iter().any(|allowed| allowed.origin() == o.origin()))
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
			ALLOWED_CORS_HEADERS.iter().copied().map(HeaderName::from_static),
		))
		.expose_headers(ExposeHeaders::list([
			HeaderName::from_static("x-access-token"),
			HeaderName::from_static("x-request-id"),
			HeaderName::from_static("x-auth-failure"),
		]))
		.max_age(MaxAge::exact(Duration::from_secs(7200)))
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
				.layer(CompressionLayer::new())
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
				.layer(cors_layer(&global))
				.layer(IpMiddleware::new(global.config.api.incoming_request.clone()))
				.layer(CookieMiddleware)
				.layer(SessionMiddleware::new(global.clone())),
		)
}

#[derive(serde::Serialize)]
struct RootResp {
	message: &'static str,
	version: &'static str,
	commit_hash: Option<&'static str>,
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
		commit_hash: option_env!("GIT_HASH"),
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

#[tracing::instrument(name = "API", skip_all)]
pub async fn run(global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
	let server = scuffle_http::backend::tcp::TcpServerConfig::builder()
		.with_bind(global.config.api.bind)
		.build()
		.into_server();

	server
		.start(
			shared::http::MonitorAcceptor::new(
				scuffle_http::svc::axum_service(routes(global.clone())),
				SocketKind::Tcp,
				None,
			),
			global.config.api.workers,
		)
		.await
		.context("Failed to start HTTP server")?;

	server
		.wait()
		.with_context(&ctx)
		.await
		.transpose()
		.context("HTTP server failed")?;

	server.shutdown().await.context("Failed to shutdown HTTP server")?;

	Ok(())
}
