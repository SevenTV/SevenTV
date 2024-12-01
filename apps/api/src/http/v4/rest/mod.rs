use std::sync::Arc;

use axum::Router;

use crate::global::Global;

mod auth;
mod emotes;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().nest("/auth", auth::routes()).nest("/emotes", emotes::routes())
}
