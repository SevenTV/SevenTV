use std::sync::Arc;

use axum::Router;

use crate::global::Global;

mod webhook;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().nest("/webhook", webhook::routes())
}
