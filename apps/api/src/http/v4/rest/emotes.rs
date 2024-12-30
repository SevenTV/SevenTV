use std::sync::Arc;

use axum::extract::multipart::Multipart;
use axum::extract::{DefaultBodyLimit, State};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Extension, Json, Router};
use bytes::Bytes;
use hyper::StatusCode;
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
use tracing::Instrument;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::validators;
use crate::ratelimit::RateLimitRequest;
use crate::transactions::{transaction, TransactionError};

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", post(create_emote).layer(DefaultBodyLimit::max(7 * 1024 * 1024)))
}

struct CreateEmoteData {
	file: Bytes,
	metadata: CreateEmoteMetadata,
}

#[derive(Debug, serde::Deserialize)]
struct CreateEmoteMetadata {
	name: String,
	tags: Vec<String>,
	default_zero_width: Option<bool>,
	private: Option<bool>,
}

async fn parse_multipart(mut multipart: Multipart) -> Result<CreateEmoteData, ApiError> {
	let mut file = None;
	let mut metadata = None;

	while let Some(field) = multipart
		.next_field()
		.await
		.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "failed to parse multipart body"))?
	{
		let field_name = field
			.name()
			.ok_or(ApiError::bad_request(ApiErrorCode::BadRequest, "missing field name"))?;

		match field_name {
			"file" => {
				file = Some(
					field
						.bytes()
						.await
						.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "failed to read field"))?,
				);
			}
			"metadata" => {
				let metadata_bytes = field
					.bytes()
					.await
					.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "failed to read field"))?;

				metadata = Some(
					serde_json::from_slice(&metadata_bytes)
						.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid metadata"))?,
				);
			}
			_ => {
				return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "unknown field"));
			}
		}
	}

	Ok(CreateEmoteData {
		file: file.ok_or(ApiError::bad_request(ApiErrorCode::BadRequest, "missing data"))?,
		metadata: metadata.ok_or(ApiError::bad_request(ApiErrorCode::BadRequest, "missing metadata"))?,
	})
}

#[derive(serde::Serialize)]
struct CreateEmoteResponse {
	emote_id: EmoteId,
}

#[tracing::instrument(skip_all)]
pub async fn create_emote(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
	multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
	let data = parse_multipart(multipart).await?;

	let authed_user = session.user()?;

	if !session.has(EmotePermission::Upload) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you do not have permission to upload emotes",
		));
	}

	if !validators::check_emote_name(&data.metadata.name) {
		return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "invalid emote name"));
	}

	if !validators::check_tags(&data.metadata.tags) {
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
			.upload_emote(emote_id, data.file, Some(session.ip()))
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
		if data.metadata.default_zero_width == Some(true) {
			flags |= EmoteFlags::DefaultZeroWidth;
		}
		if data.metadata.private == Some(true) {
			flags |= EmoteFlags::Private;
		}

		let res = transaction(&global, |mut tx| async move {
			let emote = Emote {
				id: emote_id,
				owner_id: authed_user.id,
				default_name: data.metadata.name,
				tags: data.metadata.tags,
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
			Ok(emote) => Ok((StatusCode::CREATED, Json(CreateEmoteResponse { emote_id: emote.id }))),
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
