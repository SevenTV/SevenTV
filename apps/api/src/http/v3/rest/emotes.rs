use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper::{HeaderMap, StatusCode};
use image_processor::{ProcessImageResponse, ProcessImageResponseUploadInfo};
use scuffle_image_processor_proto as image_processor;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteData, AuditLogId};
use shared::database::emote::{Emote, EmoteFlags, EmoteId};
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::role::permissions::{EmotePermission, FlagPermission, PermissionsExt};
use shared::database::MongoCollection;
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
	auth_session: Option<AuthSession>,
	headers: HeaderMap,
	body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
	let emote_data = headers.get("X-Emote-Data").ok_or(ApiError::BAD_REQUEST)?;

	let emote_data = serde_json::from_str::<XEmoteData>(emote_data.to_str().map_err(|_| ApiError::BAD_REQUEST)?)
		.map_err(|_| ApiError::BAD_REQUEST)?;

	// TODO: validate emote name
	// 	 We have automod rules too!!

	let auth_session = auth_session.ok_or(ApiError::UNAUTHORIZED)?;
	let user = auth_session.user(&global).await?;

	if !user.has(EmotePermission::Upload) {
		return Err(ApiError::FORBIDDEN);
	}

	let emote_id = EmoteId::new();

	let input = match global.image_processor.upload_emote(emote_id, body).await {
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
		owner_id: user.id,
		default_name: emote_data.name,
		tags: emote_data.tags,
		image_set: ImageSet { input, outputs: vec![] },
		flags,
		attribution: vec![],
		merged: None,
		aspect_ratio: -1.0,
		scores: Default::default(),
		search_updated_at: None,
		updated_at: chrono::Utc::now(),
	};

	let mut session = global.mongo.start_session().await.map_err(|e| {
		tracing::error!(error = %e, "failed to start session");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	session.start_transaction().await.map_err(|e| {
		tracing::error!(error = %e, "failed to start transaction");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	Emote::collection(&global.db)
		.insert_one(&emote)
		.session(&mut session)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to insert emote");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	AuditLog::collection(&global.db)
		.insert_one(AuditLog {
			id: AuditLogId::new(),
			actor_id: Some(user.id),
			data: AuditLogData::Emote {
				target_id: emote.id,
				data: AuditLogEmoteData::Upload,
			},
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		})
		.session(&mut session)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to insert audit log");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	session.commit_transaction().await.map_err(|e| {
		tracing::error!(error = %e, "failed to commit transaction");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	// we don't have to return the owner here
	let emote = EmotePartialModel::from_db(emote, None, &global.config.api.cdn_origin);

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
	auth_session: Option<Extension<AuthSession>>,
) -> Result<impl IntoResponse, ApiError> {
	let emote = global
		.emote_by_id_loader
		.load(id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found"))?;

	let owner = global
		.user_loader
		.load_fast(&global, emote.owner_id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let actor_id = auth_session.as_ref().map(|s| s.user_id());
	let can_view_hidden = if let Some(session) = &auth_session {
		session.can_view_hidden(&global).await?
	} else {
		false
	};

	let owner = owner
		.and_then(|owner| {
			if owner.has(FlagPermission::Hidden) && Some(owner.id) != actor_id && !can_view_hidden {
				None
			} else {
				Some(owner)
			}
		})
		.map(|owner| UserPartialModel::from_db(owner, None, None, &global.config.api.cdn_origin));

	Ok(Json(EmoteModel::from_db(emote, owner, &global.config.api.cdn_origin)))
}
