use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;
use utoipa::ToSchema;

use crate::{global::Global, http::error::ApiError};

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .get("/extension", get_extension)
        .get("/extension-nightly", get_extension_nightly)
}


#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ExtensionConfig {}

#[utoipa::path(
    get,
    path = "/v3/config/extension",
    responses(
        (status = 200, description = "Extension Config", body = ExtensionConfig, content_type = "application/json"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/config/config.root.go#L29
pub async fn get_extension(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!("get_extensions")
}

#[utoipa::path(
    get,
    path = "/v3/config/extension-nightly",
    responses(
        (status = 200, description = "Extension Config Nightly", body = ExtensionConfig, content_type = "application/json"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/config/config.root.go#L29
pub async fn get_extension_nightly(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!("get_extensions_nightly")
}
