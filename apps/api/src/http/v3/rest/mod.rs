use std::sync::Arc;

use axum::Router;

use crate::global::Global;

pub mod auth;
pub mod config;
pub mod emote_sets;
pub mod emotes;
pub mod entitlements;
pub mod types;
pub mod users;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.nest("/config", config::routes())
		.nest("/auth", auth::routes())
		.nest("/emotes", emotes::routes())
		.nest("/emote-sets", emote_sets::routes())
		.nest("/users", users::routes())
		.nest("/entitlements", entitlements::routes())
}
