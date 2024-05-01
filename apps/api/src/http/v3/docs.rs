use std::sync::Arc;

use axum::response::Response;
use axum::routing::get;
use axum::Router;
use hyper::body::Bytes;

use crate::global::Global;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_docs))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", get(get_docs))
}

#[utoipa::path(
    get,
    path = "/v3/docs",
    tag = "docs",
    responses(
        (status = 200, description = "Returns swagger documentation", content_type = "application/json"),
    ),
)]
#[tracing::instrument(level = "info")]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/docs/docs.go#L24
pub async fn get_docs() -> Response {
	Response::builder()
		.header(hyper::header::CONTENT_TYPE, "application/json")
		.body(memoize_docs().into())
		.unwrap()
}

// This allows us to only generate the OpenAPI documentation once and cache it
// in memory for the rest of the application's lifetime.
fn memoize_docs() -> Bytes {
	static CACHE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();

	Bytes::from_static(CACHE.get_or_init(|| {
		let docs = super::docs().to_json().unwrap();
		docs.into_bytes()
	}))
}
