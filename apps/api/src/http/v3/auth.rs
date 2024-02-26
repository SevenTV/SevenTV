use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;

use crate::{global::Global, http::error::ApiError};

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
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
pub async fn manual(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}
