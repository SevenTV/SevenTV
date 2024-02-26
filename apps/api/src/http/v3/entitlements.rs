use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;

use crate::{global::Global, http::error::ApiError};

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .get("/", get_entitlements)
        .post("/", create_entitlement)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Entitlement {}

#[utoipa::path(
    get,
    path = "/v3/entitlements",
    responses(
        (status = 200, description = "Entitlements", body = Vec<Entitlement>),
    ),
)]
pub async fn get_entitlements(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}


#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEntitlementData {}

#[utoipa::path(
    post,
    path = "/v3/entitlements",
    request_body = XEntitlementData,
    responses(
        (status = 201, description = "Entitlement Created"),
    ),
)]
pub async fn create_entitlement(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}