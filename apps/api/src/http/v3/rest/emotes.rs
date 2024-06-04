use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper::{HeaderMap, StatusCode};
use image_processor::{ProcessImageResponse, ProcessImageResponseUploadInfo};
use scuffle_image_processor_proto as image_processor;
use shared::database::{Collection, Emote, EmoteFlags, EmoteId, EmotePermission, ImageSet, ImageSetInput};
use shared::old_types::{EmoteFlagsModel, UserPartialModel};

use super::types::{EmoteModel, EmotePartialModel};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_emote, get_emote_by_id), components(schemas(XEmoteData)))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/", post(create_emote))
		.route("/:id", get(get_emote_by_id))
}

#[derive(Debug, serde::Serialize, Default, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.create.go#L385
pub struct XEmoteData {
	name: String,
	tags: Vec<String>,
	flags: EmoteFlagsModel,
}

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
pub async fn create_emote(
	State(global): State<Arc<Global>>,
	auth_session: Option<Extension<AuthSession>>,
	headers: HeaderMap,
	body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
	let emote_data = headers.get("X-Emote-Data").ok_or(ApiError::BAD_REQUEST)?;

	let emote_data = serde_json::from_str::<XEmoteData>(emote_data.to_str().map_err(|_| ApiError::BAD_REQUEST)?)
		.map_err(|_| ApiError::BAD_REQUEST)?;

	// TODO: validate emote name

	let user_id = auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id();

	let user = global
		.user_by_id_loader()
		.load(&global, user_id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::UNAUTHORIZED)?;

	let global_config = global
		.global_config_loader()
		.load(())
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	let roles = {
		let mut roles = global
			.role_by_id_loader()
			.load_many(user.entitled_cache.role_ids.iter().copied())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		global_config
			.role_ids
			.iter()
			.filter_map(|id| roles.remove(id))
			.collect::<Vec<_>>()
	};

	let permissions = user.compute_permissions(&roles);

	if !permissions.has(EmotePermission::Upload) {
		return Err(ApiError::FORBIDDEN);
	}

	let emote_id = EmoteId::new();

	let input = match global.image_processor().upload_emote(emote_id, body).await {
		Ok(ProcessImageResponse {
			id,
			error: None,
			upload_info: Some(ProcessImageResponseUploadInfo {
				path: Some(path),
				content_type,
				size,
			}),
		}) => ImageSetInput::Pending {
			task_id: id,
			path: path.path,
			mime: content_type,
			size,
		},
		Ok(ProcessImageResponse { error: Some(err), .. }) => {
			// At this point if we get a decode error then the image is invalid
			// and we should return a bad request
			if err.code == image_processor::ErrorCode::Decode as i32
				|| err.code == image_processor::ErrorCode::InvalidInput as i32
			{
				return Err(ApiError::BAD_REQUEST);
			}

			tracing::error!(code = ?err.code(), "failed to upload emote: {}", err.message);
			return Err(ApiError::INTERNAL_SERVER_ERROR);
		}
		Err(err) => {
			tracing::error!("failed to upload emote: {:#}", err);
			return Err(ApiError::INTERNAL_SERVER_ERROR);
		}
		_ => {
			tracing::error!("failed to upload emote: unknown error");
			return Err(ApiError::INTERNAL_SERVER_ERROR);
		}
	};

	let mut flags = EmoteFlags::default();
	if emote_data.flags.contains(EmoteFlagsModel::ZeroWidth) {
		flags |= EmoteFlags::DefaultZeroWidth;
	}
	if emote_data.flags.contains(EmoteFlagsModel::Private) {
		flags |= EmoteFlags::Private;
	}

	let emote = Emote {
		id: emote_id,
		owner_id: Some(user_id),
		default_name: emote_data.name,
		tags: emote_data.tags,
		animated: false, // will be set by the image processor callback
		image_set: ImageSet { input, outputs: vec![] },
		flags,
		attribution: vec![],
		replaced_by: None,
	};

	Emote::collection(global.db())
		.insert_one(emote.clone(), None)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to insert emote");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	// we don't have to return the owner here
	let emote = EmotePartialModel::from_db(emote, None, &global.config().api.cdn_base_url);

	Ok((StatusCode::CREATED, Json(emote)))
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
				.map(|u| (u, conns))
		}
		None => None,
	};

	let owner =
		owner.map(|(owner, conns)| UserPartialModel::from_db(owner, conns, None, None, &global.config().api.cdn_base_url));

	Ok(Json(EmoteModel::from_db(emote, owner, &global.config().api.cdn_base_url)))
}
