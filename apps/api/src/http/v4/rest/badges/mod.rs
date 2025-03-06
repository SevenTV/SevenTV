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
use shared::database::badge::{Badge, BadgeId};
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::role::permissions::{AdminPermission, PermissionsExt, RateLimitResource};
use shared::database::stored_event::StoredEventBadgeData;
use shared::event::{InternalEvent, InternalEventData};
use tracing::Instrument;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::ratelimit::RateLimitRequest;
use crate::transactions::{transaction, TransactionError};

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", post(create_badge).layer(DefaultBodyLimit::max(7 * 1024 * 1024)))
}

struct CreateBadgeData {
	file: Bytes,
	metadata: CreateBadgeMetadata,
}

#[derive(Debug, serde::Deserialize)]
struct CreateBadgeMetadata {
	name: String,
	description: String,
}

async fn parse_multipart(mut multipart: Multipart) -> Result<CreateBadgeData, ApiError> {
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

	Ok(CreateBadgeData {
		file: file.ok_or(ApiError::bad_request(ApiErrorCode::BadRequest, "missing data"))?,
		metadata: metadata.ok_or(ApiError::bad_request(ApiErrorCode::BadRequest, "missing metadata"))?,
	})
}

#[derive(serde::Serialize)]
struct CreateBadgeResponse {
	badge_id: BadgeId,
}

#[tracing::instrument(skip_all)]
pub async fn create_badge(
	State(global): State<Arc<Global>>,
	Extension(session): Extension<Session>,
	multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
	let data = parse_multipart(multipart).await?;

	let authed_user = session.user()?;

	if !session.has(AdminPermission::Admin) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"you do not have permission to upload badges",
		));
	}

	let req = RateLimitRequest::new(RateLimitResource::ProfilePictureUpload, &session);

	req.http(&global, async {
		let session = &session;

		let badge_id = BadgeId::new();

		let input = match global
			.image_processor
			.upload_badge(badge_id, data.file)
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

		let res = transaction(&global, |mut tx| async move {
			let badge = Badge {
				id: badge_id,
				name: data.metadata.name,
				image_set: ImageSet { input, outputs: vec![] },
				description: Some(data.metadata.description),
				tags: vec![],
				created_by_id: authed_user.id,
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			};

			tx.insert_one::<Badge>(&badge, None).await?;

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session().map(|s| s.id),
				data: InternalEventData::Badge {
					after: badge.clone(),
					data: StoredEventBadgeData::Create,
				},
				timestamp: chrono::Utc::now(),
			})?;

			Ok(badge)
		})
		.await;

		match res {
			Ok(badge) => Ok((StatusCode::CREATED, Json(CreateBadgeResponse { badge_id: badge.id }))),
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
