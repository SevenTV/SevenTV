use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use hyper::StatusCode;
use shared::database::EmoteSetId;
use shared::old_types::UserPartialModel;
use utoipa::OpenApi;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;
use crate::http::v3::emote_set_loader::load_emote_set;

use super::types::EmoteSetModel;

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
) -> Result<impl IntoResponse, ApiError> {
	let emote_set = global
		.emote_set_by_id_loader()
		.load(id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote set not found"))?;

	let emote_set_emotes = global
		.emote_set_emote_by_id_loader()
		.load(id)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default();

	let emotes = load_emote_set(&global, emote_set_emotes).await?;

	let owner = match emote_set.owner_id {
		Some(owner) => {
			let conns = global
				.user_connection_by_user_id_loader()
				.load(owner)
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.unwrap_or_default();
			global
				.user_by_id_loader()
				.load(&global, owner)
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.map(|u| UserPartialModel::from_db(u, conns, None, None, &global.config().api.cdn_base_url))
		}
		None => None,
	};

	Ok(Json(EmoteSetModel::from_db(emote_set, emotes, owner)))
}
