use std::sync::Arc;

use axum::Router;

use crate::global::Global;

mod webhooks;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().nest("/webhook", webhooks::routes())
}
