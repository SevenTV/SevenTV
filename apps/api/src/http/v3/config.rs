use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_extension_config), components(schemas(ExtensionConfig)))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/:name", get(get_extension_config))
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
#[tracing::instrument]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/config/config.root.go#L29
async fn get_extension_config(Path(name): Path<String>) -> ApiError {
	// TODO(troy): Implement this
	ApiError::NOT_IMPLEMENTED
}
