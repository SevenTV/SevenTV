use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::Body;

use crate::global::Global;
use crate::http::error::ApiError;

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_entitlement,), components(schemas(XEntitlementData),))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder().post("/", create_entitlement)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEntitlementData {}

#[utoipa::path(
    post,
    path = "/v3/entitlements",
    tag = "entitlements",
    request_body = XEntitlementData,
    responses(
        (status = 201, description = "Entitlement Created"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/entitlements/entitlements.create.go#L34
pub async fn create_entitlement(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	todo!()
}
