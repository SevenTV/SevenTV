use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper::{HeaderMap, StatusCode};
use scuffle_image_processor_proto as image_processor;
use shared::database::{Collection, Emote, EmoteFlags, EmoteId, EmotePermission, FeaturePermission, UserSession};
use axum::{Json, Router};
use hyper::StatusCode;
use shared::database::EmoteId;

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

#[derive(Debug, serde::Serialize, Default, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.create.go#L385
pub struct XEmoteData {
	name: String,
	description: String,
	tags: Vec<String>,
	zero_width: bool,
	private: bool,
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
	user_session: Option<Extension<UserSession>>,
	headers: HeaderMap,
	body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
	let emote_data = headers.get("X-Emote-Data").ok_or(ApiError::BAD_REQUEST)?;

	let emote_data = serde_json::from_str::<XEmoteData>(
		emote_data
			.to_str()
			.map_err(|_| ApiError::BAD_REQUEST)?
	).map_err(|_| ApiError::BAD_REQUEST)?;

	// TODO: validate emote name
	
	let user_session = user_session.ok_or(ApiError::UNAUTHORIZED)?.0;

	let user = global
		.user_by_id_loader()
		.load(&global, user_session.user_id)
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

	let emote = Emote {
		id: emote_id,
		owner_id: Some(user_session.user_id),
		default_name: emote_data.name,
		tags: emote_data.tags,
		flags: {
			let mut flags = EmoteFlags::default();

			flags |= if emote_data.zero_width {
				EmoteFlags::DefaultZeroWidth
			} else {
				EmoteFlags::none()
			};

			flags |= if emote_data.private {
				EmoteFlags::Private
			} else {
				EmoteFlags::none()
			};

			flags
		},
		..Default::default()
	};

	let mut session = global.mongo().start_session(None).await.map_err(|err| {
		tracing::error!(error = %err, "failed to start session");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	Emote::collection(global.db())
		.insert_one_with_session(emote, None, &mut session)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to insert emote");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	let result = match global.image_processor().upload_emote(emote_id, body).await {
		Ok(result) => result,
		Err(err) => {
			tracing::error!("failed to upload emote: {:#}", err);
			return Err(ApiError::INTERNAL_SERVER_ERROR);
		}
	};

	if let Some(err) = result.error {
		// At this point if we get a decode error then the image is invalid
		// and we should return a bad request
		if err.code == image_processor::ErrorCode::Decode as i32 {
			return Err(ApiError::BAD_REQUEST);
		}

		tracing::error!(code = ?err.code(), "failed to upload emote: {}", err.message);
		return Err(ApiError::INTERNAL_SERVER_ERROR);
	}

	Ok(StatusCode::CREATED)
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
		owner.map(|(owner, conns)| owner.into_old_model_partial(conns, None, None, &global.config().api.cdn_base_url));

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
		.load_many(pfp_file_set_id.into_iter().chain(emote.file_set_id))
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let emote_file_set = emote.file_set_id.map(|set_id| file_sets.get(&set_id).ok_or(ApiError::INTERNAL_SERVER_ERROR)).transpose()?;

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
