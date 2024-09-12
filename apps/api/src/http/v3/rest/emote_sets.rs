use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use shared::database::emote_set::EmoteSetId;
use shared::old_types::{EmoteSetModel, UserPartialModel};
use utoipa::OpenApi;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Path;
use crate::http::middleware::session::Session;
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
	Extension(session): Extension<Session>,
) -> Result<impl IntoResponse, ApiError> {
	let mut emote_set = global
		.emote_set_by_id_loader
		.load(id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))?;

	let owner = match emote_set.owner_id {
		Some(owner_id) => global
			.user_loader
			.load_fast(&global, owner_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.and_then(|owner| session.can_view(&owner).then_some(owner))
			.map(|owner| UserPartialModel::from_db(owner, None, None, &global.config.api.cdn_origin)),
		None => None,
	};

	let emotes = load_emote_set(&global, std::mem::take(&mut emote_set.emotes), &session).await?;

	Ok(Json(EmoteSetModel::from_db(emote_set, emotes, owner)))
}
