use std::sync::Arc;

use axum::{routing::post, Router};
use utoipa::OpenApi;

use crate::global::Global;

pub mod docs;
pub mod emote_set_loader;
pub mod gql;
pub mod rest;

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
	docs.merge(gql::Docs::openapi());
	docs.merge(rest::config::Docs::openapi());
	docs.merge(rest::auth::Docs::openapi());
	docs.merge(rest::emotes::Docs::openapi());
	docs.merge(rest::emote_sets::Docs::openapi());
	docs.merge(rest::users::Docs::openapi());
	docs.merge(rest::entitlements::Docs::openapi());
	docs
}

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.nest("/", rest::routes())
		.nest("/docs", docs::routes())
		.route("/gql", post(gql::handler))
}
