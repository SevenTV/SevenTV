use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use hyper::StatusCode;
use shared::database::emote_set::EmoteSetId;
use shared::database::role::permissions::{FlagPermission, PermissionsExt};
use shared::old_types::{EmoteSetModel, UserPartialModel};
use utoipa::OpenApi;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::emote_set_loader::load_emote_set;

#[derive(OpenApi)]
#[openapi(paths(get_emote_set_by_id), components(schemas(EmoteSetModel)))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/:id", get(get_emote_set_by_id))
}

#[utoipa::path(
    get,
    path = "/v3/emote-sets/{id}",
    tag = "emote-sets",
    responses(
        (status = 200, description = "Emote Set", body = EmoteSetModel, content_type = "application/json"),
        // (status = 404, description = "Emote Set Not Found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "The ID of the emote set"),
    ),
)]
#[tracing::instrument(skip_all, fields(id = %id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emote-sets/emote-sets.by-id.go#L42
pub async fn get_emote_set_by_id(
	State(global): State<Arc<Global>>,
	Path(id): Path<EmoteSetId>,
	auth_session: Option<AuthSession>,
) -> Result<impl IntoResponse, ApiError> {
	let mut emote_set = global
		.emote_set_by_id_loader
		.load(id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote set not found"))?;

	let actor_id = auth_session.as_ref().map(|s| s.user_id());

	let owner = match emote_set.owner_id {
		Some(owner_id) => global
			.user_loader
			.load_fast(&global, owner_id)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.and_then(|owner| {
				if owner.has(FlagPermission::Hidden) && Some(owner.id) != actor_id {
					None
				} else {
					Some(owner)
				}
			})
			.map(|owner| UserPartialModel::from_db(owner, None, None, &global.config.api.cdn_origin)),
		None => None,
	};

	let emotes = load_emote_set(&global, std::mem::take(&mut emote_set.emotes), actor_id, false).await?;

	Ok(Json(EmoteSetModel::from_db(emote_set, emotes, owner)))
}
