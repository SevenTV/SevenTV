use std::sync::Arc;

use axum::Router;

use crate::global::Global;

mod auth;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().nest("/auth", auth::routes())
}
