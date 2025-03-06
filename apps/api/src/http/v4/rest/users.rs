use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use image_processor_proto::{ProcessImageResponse, ProcessImageResponseUploadInfo};
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::editor::{EditorUserPermission, UserEditorId};
use shared::database::user::profile_picture::{UserProfilePicture, UserProfilePictureId};
use shared::database::user::{User, UserId, UserStyle};
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::ratelimit::RateLimitRequest;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route(
			"/:id/profile-picture",
			post(upload_user_profile_picture).layer(DefaultBodyLimit::max(7 * 1024 * 1024)),
		)
		.route("/:id/products", get(get_user_products))
}

#[derive(serde::Serialize)]
struct UploadUserProfilePictureResponse {
	pending_profile_picture: UserProfilePictureId,
}

#[tracing::instrument(skip_all, fields(id = ?id))]
async fn upload_user_profile_picture(
	State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
	Extension(session): Extension<Session>,
	body: axum::body::Bytes,
) -> Result<impl IntoResponse, ApiError> {
	let authed_user = session.user()?;

	let target_user = global
		.user_loader
		.load(&global, id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

	if !target_user.computed.permissions.has(UserPermission::UseCustomProfilePicture) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"user cannot set custom profile picture",
		));
	}

	if target_user.id != authed_user.id
		&& !authed_user.has(UserPermission::ManageAny)
		&& !global
			.user_editor_by_id_loader
			.load(UserEditorId {
				user_id: target_user.id,
				editor_id: authed_user.id,
			})
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
			.map(|editor| editor.permissions.has_user(EditorUserPermission::ManageProfile))
			.unwrap_or_default()
	{
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
			"user cannot edit other user's profile picture",
		));
	}

	if target_user.style.pending_profile_picture.is_some() {
		return Err(ApiError::conflict(
			ApiErrorCode::MutationError,
			"profile picture change already pending",
		));
	}

	let req = RateLimitRequest::new(RateLimitResource::ProfilePictureUpload, &session);

	req.http(&global, async {
		let profile_picture_id = UserProfilePictureId::new();

		let input = match global
			.image_processor
			.upload_profile_picture(profile_picture_id, target_user.id, body, Some(session.ip()))
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
				if err.code == image_processor_proto::ErrorCode::Decode as i32
					|| err.code == image_processor_proto::ErrorCode::InvalidInput as i32
				{
					return Err(ApiError::bad_request(
						ApiErrorCode::ImageProcessorError,
						"failed to upload profile picture",
					));
				}

				tracing::error!(code = ?err.code(), "failed to upload profile picture: {}", err.message);
				return Err(ApiError::internal_server_error(
					ApiErrorCode::ImageProcessorError,
					"failed to upload profile picture",
				));
			}
			Err(err) => {
				tracing::error!("failed to upload profile picture: {:#}", err);
				return Err(ApiError::internal_server_error(
					ApiErrorCode::ImageProcessorError,
					"failed to upload profile picture",
				));
			}
			_ => {
				tracing::error!("failed to upload profile picture: unknown error");
				return Err(ApiError::internal_server_error(
					ApiErrorCode::ImageProcessorError,
					"failed to upload profile picture",
				));
			}
		};

		UserProfilePicture::collection(&global.db)
			.insert_one(UserProfilePicture {
				id: profile_picture_id,
				user_id: target_user.id,
				image_set: ImageSet { input, outputs: vec![] },
				updated_at: chrono::Utc::now(),
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert profile picture");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to insert profile picture")
			})?;

		User::collection(&global.db)
			.update_one(
				filter::filter! {
					User {
						#[query(rename = "_id")]
						id: target_user.id,
					}
				},
				update::update! {
					#[query(set)]
					User {
						#[query(flatten)]
						style: UserStyle {
							pending_profile_picture: Some(profile_picture_id),
						},
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					}
				},
			)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to update user");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update user")
			})?;

		Ok(Json(UploadUserProfilePictureResponse {
			pending_profile_picture: profile_picture_id,
		}))
	})
	.await
}

#[tracing::instrument(skip_all, fields(id = ?id))]
async fn get_user_products(
	State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError> {
	let user = global
		.user_loader
		.load(&global, id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

	let owned_products: Vec<_> = user.computed.entitlements.products.into_iter().collect();
	Ok(Json(owned_products))
}
