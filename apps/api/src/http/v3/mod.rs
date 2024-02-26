use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;
use utoipa::OpenApi;

use crate::global::Global;

use super::error::ApiError;

pub mod docs;
pub mod config;
pub mod auth;
pub mod emotes;
pub mod emote_sets;
pub mod users;
pub mod entitlements;

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
    docs.merge(docs::Docs::openapi());
    docs.merge(config::Docs::openapi());
    docs.merge(auth::Docs::openapi());
    docs.merge(emotes::Docs::openapi());
    docs.merge(emote_sets::Docs::openapi());
    docs.merge(users::Docs::openapi());
    docs.merge(entitlements::Docs::openapi());
    docs
}

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .scope("/docs", docs::routes(global))
        .scope("/config", config::routes(global))
        .scope("/auth", auth::routes(global))
        .scope("/emotes", emotes::routes(global))
        .scope("/emote-sets", emote_sets::routes(global))
        .scope("/users", users::routes(global))
        .scope("/entitlements", entitlements::routes(global))
}
