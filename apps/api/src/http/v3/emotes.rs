use std::sync::Arc;

use axum::extract::{Path, Request, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use hyper::StatusCode;
use shared::database::{EmoteId, FeaturePermission};

use crate::global::Global;
use crate::http::error::ApiError;

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_emote, get_emote_by_id), components(schemas(XEmoteData)))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/", post(create_emote))
		.route("/:id", get(get_emote_by_id))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEmoteData {}

#[utoipa::path(
    post,
    path = "/v3/emotes",
    tag = "emotes",
    // Currently utoipa does not support multiple request body types so we use `image/*` as a placeholder
    // See https://github.com/juhaku/utoipa/pull/876
    request_body(content = &[u8], description = "Image Binary Data", content_type = "image/*"),
    responses(
        (status = 201, description = "Emote Created"),
    ),
    params(
        ("X-Emote-Data" = XEmoteData, Header, description = "The properties of the emote"),
    ),
)]
#[tracing::instrument(skip(global))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.create.go#L58
pub async fn create_emote(State(global): State<Arc<Global>>, req: Request) -> Result<impl IntoResponse, ApiError> {
	let _ = global;
	Ok(ApiError::NOT_IMPLEMENTED)
}

#[utoipa::path(
    get,
    path = "/v3/emotes/{id}",
    tag = "emotes",
    responses(
        (status = 200, description = "Emote", body = EmoteModel),
        (status = 404, description = "Emote Not Found")
    ),
    params(
        ("id" = String, Path, description = "The ID of the emote"),
    ),
)]
#[tracing::instrument(skip_all, fields(id = %id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.by-id.go#L36
pub async fn get_emote_by_id(
	State(global): State<Arc<Global>>,
	Path(id): Path<EmoteId>,
) -> Result<impl IntoResponse, ApiError> {
	let emote = global
		.emote_by_id_loader()
		.load(id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found"))?;

	let owner = match emote.owner_id {
		Some(owner) => global
			.user_by_id_loader()
			.load(&global, owner)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?,
		None => None,
	};

	let pfp_file_set_id = match (&owner, owner.as_ref().and_then(|owner| owner.style.active_profile_picture_id)) {
		(Some(owner), Some(profile_picture_id)) => {
			let roles = global
				.role_by_id_loader()
				.load_many(owner.entitled_cache.role_ids.iter().copied())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

			let global_config = global
				.global_config_loader()
				.load(())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

			let roles = global_config
				.role_ids
				.iter()
				.filter_map(|id| roles.get(id))
				.cloned()
				.collect::<Vec<_>>();

			if owner
				.compute_permissions(&roles)
				.has(FeaturePermission::UseCustomProfilePicture)
			{
				Some(profile_picture_id)
			} else {
				None
			}
		}
		_ => None,
	};

	let file_sets = global
		.file_set_by_id_loader()
		.load_many(pfp_file_set_id.into_iter().chain(Some(emote.file_set_id)))
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let emote_file_set = file_sets.get(&emote.file_set_id).ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	let owner = owner.map(|owner| {
		owner.into_old_model_partial(
			Vec::new(),
			pfp_file_set_id.and_then(|id| file_sets.get(&id)),
			None,
			None,
			&global.config().api.cdn_base_url,
		)
	});

	Ok(Json(emote.into_old_model(
		owner,
		emote_file_set,
		&global.config().api.cdn_base_url,
	)))
}
