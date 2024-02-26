use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;

use crate::{global::Global, http::error::ApiError};

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .get("/", root)
        .post("/logout", logout)
        .get("/manual", manual)
}

#[utoipa::path(
    get,
    path = "/v3/auth",
    responses(
        (status = 307, description = "Auth Redirect"),
    ),
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/auth.route.go#L47
pub async fn root(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    post,
    path = "/v3/auth/logout",
    responses(
        (status = 204, description = "Logout"),
    ),
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/logout.auth.route.go#L29
pub async fn logout(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/auth/manual",
    responses(
        (status = 200, description = "Manual Auth"),
    ),
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/auth/manual.route.go#L41
pub async fn manual(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}
