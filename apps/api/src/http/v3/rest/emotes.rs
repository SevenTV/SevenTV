use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper::{HeaderMap, StatusCode};
use image_processor::{ProcessImageResponse, ProcessImageResponseUploadInfo};
use image_processor_proto as image_processor;
use shared::database::emote::{Emote, EmoteFlags, EmoteId};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::queries::filter;
use shared::database::role::permissions::{EmotePermission, PermissionsExt, RateLimitResource};
use shared::database::stored_event::StoredEventEmoteData;
use shared::database::MongoCollection;
use shared::event::{InternalEvent, InternalEventData};
use shared::old_types::{EmoteFlagsModel, EmotePartialModel, UserPartialModel};
use tracing::Instrument;

use super::types::EmoteModel;
use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::validators;
use crate::ratelimit::RateLimitRequest;
use crate::transactions::{transaction, TransactionError};

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
#[tracing::instrument(skip_all)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.create.go#L58
pub async fn create_emote(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
	headers: HeaderMap,
	body: axum::body::Body,
) -> Result<impl IntoResponse, ApiError> {
	let body = axum::body::to_bytes(body, 7 * 1024 * 1024).await.map_err(|e| {
		tracing::warn!(error = %e, "body too large");
		ApiError::bad_request(ApiErrorCode::BadRequest, "body too large")
	})?;

	let authed_user = session.user()?;

	if !session.has(EmotePermission::Upload) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you do not have permission to upload emotes",
		));
	}

	let emote_data = headers
		.get("X-Emote-Data")
		.ok_or_else(|| ApiError::bad_request(ApiErrorCode::BadRequest, "missing X-Emote-Data header"))?;

	let emote_data = serde_json::from_str::<XEmoteData>(
		emote_data
			.to_str()
			.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid X-Emote-Data header"))?,
	)
	.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid X-Emote-Data header"))?;

	if !validators::check_emote_name(&emote_data.name) {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "invalid emote name"));
	}

	if !validators::check_tags(&emote_data.tags) {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "invalid tags"));
	}

	let req = RateLimitRequest::new(RateLimitResource::ProfilePictureUpload, &session);

	req.http(&global, async {
		let count = EmoteModerationRequest::collection(&global.db)
			.count_documents(filter::filter! {
				EmoteModerationRequest {
					#[query(serde)]
					kind: EmoteModerationRequestKind::PublicListing,
					user_id: authed_user.id,
					#[query(serde)]
					status: EmoteModerationRequestStatus::Pending,
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to count emote moderation requests");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to count emote moderation requests")
			})?;

		if count as i32
			> authed_user
				.computed
				.permissions
				.emote_moderation_request_limit
				.unwrap_or_default()
		{
			return Err(ApiError::bad_request(
				ApiErrorCode::LackingPrivileges,
				"too many pending moderation requests",
			));
		}

		let session = &session;

		let emote_id = EmoteId::new();

		let input = match global
			.image_processor
			.upload_emote(emote_id, body, Some(session.ip()))
			.instrument(tracing::info_span!("image_processor_upload"))
			.await
		{
			Ok(ProcessImageResponse {
				id,
				error: None,
				upload_info:
					Some(ProcessImageResponseUploadInfo {
						path: Some(path),
						content_type,
						size,
					}),
			}) => ImageSetInput::Pending {
				task_id: id,
				path: path.path,
				mime: content_type,
				size: size as i64,
			},
			Ok(ProcessImageResponse { error: Some(err), .. }) => {
				// At this point if we get a decode error then the image is invalid
				// and we should return a bad request
				if err.code == image_processor::ErrorCode::Decode as i32
					|| err.code == image_processor::ErrorCode::InvalidInput as i32
				{
					return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "bad image format"));
				}

				tracing::error!(code = ?err.code(), "failed to upload emote: {}", err.message);
				return Err(ApiError::internal_server_error(
					ApiErrorCode::ImageProcessorError,
					"failed to upload emote",
				));
			}
			Err(err) => {
				tracing::error!("failed to upload emote: {:#}", err);
				return Err(ApiError::internal_server_error(
					ApiErrorCode::ImageProcessorError,
					"failed to upload emote",
				));
			}
			_ => {
				tracing::error!("failed to upload emote: unknown error");
				return Err(ApiError::internal_server_error(
					ApiErrorCode::ImageProcessorError,
					"failed to upload emote",
				));
			}
		};

		let mut flags = EmoteFlags::default();
		if emote_data.flags.contains(EmoteFlagsModel::ZeroWidth) {
			flags |= EmoteFlags::DefaultZeroWidth;
		}
		if emote_data.flags.contains(EmoteFlagsModel::Private) {
			flags |= EmoteFlags::Private;
		}

		let res = transaction(&global, |mut tx| async move {
			let emote = Emote {
				id: emote_id,
				owner_id: authed_user.id,
				default_name: emote_data.name,
				tags: emote_data.tags,
				image_set: ImageSet { input, outputs: vec![] },
				flags,
				attribution: vec![],
				merged: None,
				aspect_ratio: -1.0,
				scores: Default::default(),
				deleted: false,
				search_updated_at: None,
				updated_at: chrono::Utc::now(),
			};

			tx.insert_one::<Emote>(&emote, None).await?;

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session().map(|s| s.id),
				data: InternalEventData::Emote {
					after: emote.clone(),
					data: StoredEventEmoteData::Upload,
				},
				timestamp: chrono::Utc::now(),
			})?;

			Ok(emote)
		})
		.await;

		match res {
			Ok(emote) => {
				// we don't have to return the owner here
				let emote = EmotePartialModel::from_db(emote, None, &global.config.api.cdn_origin);
				Ok((StatusCode::CREATED, Json(emote)))
			}
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	})
	.await
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
	Extension(session): Extension<Session>,
) -> Result<impl IntoResponse, ApiError> {
	let emote = global
		.emote_by_id_loader
		.load_exclude_deleted(id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))?;

	let owner = global
		.user_loader
		.load_fast(&global, emote.owner_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

	let owner = owner
		.and_then(|owner| session.can_view(&owner).then_some(owner))
		.map(|owner| UserPartialModel::from_db(owner, None, None, &global.config.api.cdn_origin));

	Ok(Json(EmoteModel::from_db(emote, owner, &global.config.api.cdn_origin)))
}
