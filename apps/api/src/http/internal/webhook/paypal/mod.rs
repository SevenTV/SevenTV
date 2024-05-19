use std::sync::Arc;

use axum::Router;

use crate::global::Global;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
}
