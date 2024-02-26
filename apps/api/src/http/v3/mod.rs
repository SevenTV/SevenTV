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

#[derive(OpenApi)]
#[openapi(
    paths(
        docs::get_docs,
        config::get_extension,
        config::get_extension_nightly,
        auth::root,
        auth::logout,
        auth::manual,
        emotes::get_emote_by_id,
        emotes::create_emote,
        emote_sets::get_emote_set_by_id,
        users::get_user_by_id,
        users::get_user_profile_picture_by_id,
        users::get_user_presences_by_platform,
        users::get_user_by_platform_user_id,
        users::delete_user_by_id,
        users::update_user_connection_by_id,
        entitlements::create_entitlement,
    ),
    components(
        schemas(config::ExtensionConfig),
        schemas(emotes::XEmoteData),
        schemas(emotes::Emote),
        schemas(emote_sets::EmoteSet),
        schemas(users::User),
        schemas(users::UserConnection),
        schemas(users::UserProfilePicture),
        schemas(entitlements::Entitlement),
        schemas(entitlements::XEntitlementData),
    ),
)]
struct ApiDoc;

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
