use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;

use crate::global::Global;

mod rest;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.nest("/", rest::routes())
		.layer(CorsLayer::permissive())
}
