use std::sync::Arc;

use axum::Router;
use utoipa::OpenApi;

use crate::global::Global;

pub mod auth;
pub mod config;
pub mod docs;
pub mod emote_sets;
pub mod emotes;
pub mod entitlements;
pub mod users;

pub fn docs() -> utoipa::openapi::OpenApi {
	#[derive(OpenApi)]
	#[openapi(
        info(
            title = "7TV",
            version = "3.0.0",
            contact(
                email = "support@7tv.app",
            ),
            license(
                name = "Apache 2.0 with Commons Clause",
                url = "https://github.com/SevenTV/SevenTV/blob/main/licenses/CC_APACHE2_LICENSE",
            ),
            description = include_str!("DESCRIPTION.md"),
        ),
        servers(
            (url = "https://7tv.io", description = "Production"),
        ),
    )]
	struct Docs;

	let mut docs = Docs::openapi();
	docs.merge(shared::types::old::Docs::openapi());
	docs.merge(docs::Docs::openapi());
	docs.merge(config::Docs::openapi());
	docs.merge(auth::Docs::openapi());
	docs.merge(emotes::Docs::openapi());
	docs.merge(emote_sets::Docs::openapi());
	docs.merge(users::Docs::openapi());
	docs.merge(entitlements::Docs::openapi());
	docs
}

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.nest("/docs", docs::routes())
		.nest("/config", config::routes())
		.nest("/auth", auth::routes())
		.nest("/emotes", emotes::routes())
		.nest("/emote-sets", emote_sets::routes())
		.nest("/users", users::routes())
		.nest("/entitlements", entitlements::routes())
}
