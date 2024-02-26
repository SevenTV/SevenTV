use std::sync::Arc;

use hyper::{body::{Bytes, Incoming}, Response};
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;
use utoipa::ToSchema;

use crate::{global::Global, http::error::ApiError};

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
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
pub async fn get_extension_nightly(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!("get_extensions_nightly")
}
