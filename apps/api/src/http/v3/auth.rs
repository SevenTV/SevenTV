use std::str::FromStr;
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::{Response, StatusCode};
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::{empty_body, Body};
use shared::database::{Platform, UserSession};

use self::login::{handle_callback as handle_login_callback, handle_login};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::AUTH_COOKIE;
use crate::http::{RequestGlobalExt, RequestQueryParamExt};

mod login;

#[derive(utoipa::OpenApi)]
#[openapi(paths(login, logout, manual))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder()
		.get("/", login)
		.post("/logout", logout)
		.get("/manual", manual)
}

#[utoipa::path(
    get,
    path = "/v3/auth",
    tag = "auth",
    responses(
        (status = 303, description = "Auth Redirect"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/auth.route.go#L47
async fn login(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;

	let cookies = req.get_cookies()?;

	let platform = req
		.query_param("platform")
		.and_then(|p| Platform::from_str(&p).ok())
		.map_err_route((StatusCode::BAD_REQUEST, "invalid account provider"))?;

	let callback = req.query_param("callback").is_some_and(|c| c == "true");

	Response::builder()
		.header(
			hyper::header::LOCATION,
			if callback {
				handle_login_callback(&global, platform, &req, &cookies).await?
			} else {
				handle_login(&global, platform, &cookies)?
			},
		)
		.status(StatusCode::SEE_OTHER)
		.body(empty_body())
		.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to build response"))
}

#[utoipa::path(
    post,
    path = "/v3/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Logout"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/logout.auth.route.go#L29
async fn logout(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;
	if let Some(session) = req.extensions().get::<UserSession>() {
		scuffle_utils::database::query("DELETE FROM user_sessions WHERE id = $1")
			.bind(session.id)
			.build()
			.execute(global.db())
			.await
			.map_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to delete session"))?;

		req.get_cookies()?.remove(AUTH_COOKIE);
	}

	Ok(Response::builder()
		.status(StatusCode::NO_CONTENT)
		.body(empty_body())
		.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to build response"))?)
}

#[utoipa::path(
    get,
    path = "/v3/auth/manual",
    tag = "auth",
    responses(
        (status = 200, description = "Manual Auth"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/manual.route.go#L41
async fn manual(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	Err((StatusCode::NOT_IMPLEMENTED, "not implemented").into())
}
