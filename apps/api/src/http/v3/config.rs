use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::Body;

use crate::global::Global;
use crate::http::error::ApiError;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_extension_config), components(schemas(ExtensionConfig)))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder().get("/:name", get_extension_config)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ExtensionConfig {}

#[utoipa::path(
    get,
    path = "/v3/config/{name}",
    tag = "config",
    responses(
        (status = 200, description = "Extension Config", body = ExtensionConfig, content_type = "application/json"),
    ),
    params(
        ("name" = String, Path, description = "The name of the extension to get the config for"),
    )
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/config/config.root.go#L29
async fn get_extension_config(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	Err((StatusCode::NOT_IMPLEMENTED, "not implemented").into())
}
