use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::Body;
use shared::object_id::ObjectId;
use shared::types::old::*;
use utoipa::OpenApi;

use super::error::ApiError;
use crate::global::Global;

pub mod auth;
pub mod config;
pub mod docs;
pub mod emote_sets;
pub mod emotes;
pub mod entitlements;
pub mod types;
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
		components(schemas(
			UserModelPartial,
			ObjectId,
			UserStyle,
			CosmeticBadgeModel,
			CosmeticPaint,
			CosmeticPaintGradient,
			CosmeticPaintFunction,
			CosmeticPaintGradientStop,
			CosmeticPaintCanvasRepeat,
			CosmeticPaintShadow,
			CosmeticPaintText,
			CosmeticPaintTextTransform,
			CosmeticPaintStroke,
			UserConnectionPartial,
			ImageHost,
			ImageHostFile
		))
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
