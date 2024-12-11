use std::sync::Arc;

use axum::Router;

use crate::global::Global;

mod auth;
mod badges;
mod emotes;
mod events;
mod users;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.nest("/auth", auth::routes())
		.nest("/badges", badges::routes())
		.nest("/emotes", emotes::routes())
		.nest("/events", events::routes())
		.nest("/users", users::routes())
}
