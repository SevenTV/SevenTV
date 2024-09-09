use std::str::FromStr;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::{Extension, Json, Router};
use hyper::StatusCode;
use mongodb::bson::doc;
use scuffle_image_processor_proto::{self as image_processor, ProcessImageResponse, ProcessImageResponseUploadInfo};
use serde::Deserialize;
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{FlagPermission, PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::connection::Platform;
use shared::database::user::editor::{EditorUserPermission, UserEditorId};
use shared::database::user::profile_picture::{UserProfilePicture, UserProfilePictureId};
use shared::database::user::{User, UserId, UserStyle};
use shared::database::MongoCollection;
use shared::old_types::{
	EmoteSetModel, EmoteSetPartialModel, UserConnectionModel, UserConnectionPartialModel, UserEditorModel, UserModel,
};

use super::types::PresenceModel;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;
use crate::http::middleware::session::Session;
use crate::http::v3::emote_set_loader::load_emote_set;
use crate::ratelimit::RateLimitRequest;

#[derive(utoipa::OpenApi)]
#[openapi(
	paths(
		get_user_by_id,
		upload_user_profile_picture,
		create_user_presence,
		get_user_by_platform_id,
		delete_user_by_id,
		update_user_connection_by_id,
	),
	components(schemas())
)]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/:id", get(get_user_by_id))
		.route("/:id/profile-picture", put(upload_user_profile_picture))
		.route("/:id/presences", post(create_user_presence))
		.route("/:platform/:platform_id", get(get_user_by_platform_id))
		.route("/:id", delete(delete_user_by_id))
		.route("/:id/connections/:connection_id", patch(update_user_connection_by_id))
}

#[utoipa::path(
    get,
    path = "/v3/users/{id}",
    tag = "users",
    responses(
        (status = 200, description = "User", body = UserModel),
        // (status = 404, description = "User Not Found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "The ID of the user"),
    ),
)]
#[tracing::instrument(skip_all, fields(id = %id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.by-id.go#L44
pub async fn get_user_by_id(
	State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
	Extension(session): Extension<Session>,
) -> Result<impl IntoResponse, ApiError> {
	let user = global
		.user_loader
		.load(&global, id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::NOT_FOUND)?;

	if user.has(FlagPermission::Hidden) && Some(user.id) != session.user_id() && !session.has(UserPermission::ViewHidden) {
		return Err(ApiError::NOT_FOUND);
	}

	let emote_sets = global
		.emote_set_by_user_id_loader
		.load(user.id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default();

	let editors = global
		.user_editor_by_user_id_loader
		.load(user.id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default();

	let active_emote_set = if let Some(emote_set_id) = user.style.active_emote_set_id {
		global
			.emote_set_by_id_loader
			.load(emote_set_id)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
	} else {
		None
	};

	let mut old_model = UserModel::from_db(
		user,
		None,
		None,
		emote_sets
			.into_iter()
			.map(|emote_set| EmoteSetPartialModel::from_db(emote_set, None))
			.collect(),
		editors.into_iter().filter_map(UserEditorModel::from_db).collect(),
		&global.config.api.cdn_origin,
	);

	if let Some(mut active_emote_set) = active_emote_set {
		let emotes = load_emote_set(&global, std::mem::take(&mut active_emote_set.emotes), &session).await?;
		let model = EmoteSetModel::from_db(active_emote_set, emotes, None);

		// TODO: this seems a bit excessive im not sure if we need to do this as it
		// makes the payload very large.
		old_model.connections.iter_mut().for_each(|conn| {
			conn.emote_set = Some(model.clone());
		});
	}

	Ok(Json(old_model))
}

#[derive(Debug, Clone, Deserialize)]
pub enum TargetUser {
	#[serde(rename = "@me")]
	Me,
	#[serde(untagged)]
	Other(UserId),
}

#[utoipa::path(
    put,
    path = "/v3/users/{id}/profile-picture",
    tag = "users",
    request_body(content = &[u8], description = "Image Binary Data", content_type = "image/*"),
    responses(
        (status = 200, description = "Success"),
    ),
    params(
        ("id" = String, Path, description = "The ID of the user"),
    ),
)]
#[tracing::instrument(skip_all, fields(id = ?id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.pictures.go#L61
pub async fn upload_user_profile_picture(
	State(global): State<Arc<Global>>,
	Path(id): Path<TargetUser>,
	Extension(session): Extension<Session>,
	body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
	let authed_user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

	let other_user = match id {
		TargetUser::Me => None,
		TargetUser::Other(id) => Some(
			global
				.user_loader
				.load(&global, id)
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::NOT_FOUND)?,
		),
	};

	let target_user = other_user.as_ref().unwrap_or(authed_user);

	if target_user.computed.permissions.has(UserPermission::UseCustomProfilePicture) {
		return Err(ApiError::new_const(
			StatusCode::FORBIDDEN,
			"user cannot set custom profile picture",
		));
	}

	if other_user.is_some()
		&& !authed_user.has(UserPermission::ManageAny)
		&& !global
			.user_editor_by_id_loader
			.load(UserEditorId {
				user_id: target_user.id,
				editor_id: authed_user.id,
			})
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|editor| editor.permissions.has_user(EditorUserPermission::ManageProfile))
			.unwrap_or_default()
	{
		return Err(ApiError::new_const(
			StatusCode::FORBIDDEN,
			"user cannot edit other user's profile picture",
		));
	}

	if target_user.style.pending_profile_picture.is_some() {
		return Err(ApiError::new_const(
			StatusCode::CONFLICT,
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
				if err.code == image_processor::ErrorCode::Decode as i32
					|| err.code == image_processor::ErrorCode::InvalidInput as i32
				{
					return Err(ApiError::BAD_REQUEST);
				}

				tracing::error!(code = ?err.code(), "failed to upload profile picture: {}", err.message);
				return Err(ApiError::INTERNAL_SERVER_ERROR);
			}
			Err(err) => {
				tracing::error!("failed to upload profile picture: {:#}", err);
				return Err(ApiError::INTERNAL_SERVER_ERROR);
			}
			_ => {
				tracing::error!("failed to upload profile picture: unknown error");
				return Err(ApiError::INTERNAL_SERVER_ERROR);
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
				ApiError::INTERNAL_SERVER_ERROR
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
							active_profile_picture: Some(profile_picture_id),
						},
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to update user");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(StatusCode::OK)
	})
	.await
}

#[utoipa::path(
    post,
    path = "/v3/users/{id}/presences",
    tag = "users",
    responses(
        (status = 200, description = "User Presence", body = PresenceModel),
    ),
    params(
        ("id" = String, Path, description = "The ID of the user"),
    ),
)]
#[tracing::instrument(skip_all, fields(id = %id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.presence.write.go#L41
pub async fn create_user_presence(
	// State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
	Json(_presence): Json<PresenceModel>,
) -> Result<impl IntoResponse, ApiError> {
	// TODO: decide what to do with this
	Ok(ApiError::NOT_IMPLEMENTED)
}

#[utoipa::path(
    get,
    path = "/v3/users/{platform}/{platform_id}",
    tag = "users",
    responses(
        (status = 200, description = "User", body = UserModel),
        (status = 404, description = "User Not Found", body = ApiError)
    ),
    params(
        ("platform" = String, Path, description = "The platform"),
        ("platform_id" = String, Path, description = "The ID of the user on the platform"),
    ),
)]
#[tracing::instrument(skip_all, fields(platform = %platform, platform_id = %platform_id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.by-connection.go#L42
pub async fn get_user_by_platform_id(
	State(global): State<Arc<Global>>,
	Path((platform, platform_id)): Path<(String, String)>,
	Extension(session): Extension<Session>,
) -> Result<impl IntoResponse, ApiError> {
	let platform = Platform::from_str(&platform.to_lowercase()).map_err(|_| ApiError::BAD_REQUEST)?;

	let user = global
		.user_loader
		.load_user(
			&global,
			global
				.user_by_platform_id_loader
				.load((platform, platform_id.clone()))
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::NOT_FOUND)?,
		)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	if user.has(FlagPermission::Hidden)
		&& Some(user.id) != session.user_id()
		&& !session.permissions().has(UserPermission::ViewHidden)
	{
		return Err(ApiError::NOT_FOUND);
	}

	let connection = user
		.connections
		.iter()
		.find(|c| c.platform == platform && c.platform_id == platform_id)
		.ok_or(ApiError::NOT_FOUND)?;

	let mut connection_model: UserConnectionModel = UserConnectionPartialModel::from_db(
		connection.clone(),
		user.style.active_emote_set_id,
		user.computed.permissions.emote_set_capacity.unwrap_or_default().max(0),
	)
	.into();

	let editors = global
		.user_editor_by_user_id_loader
		.load(user.id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default()
		.into_iter()
		.filter_map(UserEditorModel::from_db)
		.collect::<Vec<_>>();

	// query user emote sets
	let emote_sets = global
		.emote_set_by_user_id_loader
		.load(user.id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default()
		.into_iter()
		.map(|s| EmoteSetPartialModel::from_db(s, None))
		.collect::<Vec<_>>();

	connection_model.emote_set_id = user.style.active_emote_set_id;

	if let Some(emote_set_id) = connection_model.emote_set_id {
		if let Some(mut emote_set) = global
			.emote_set_by_id_loader
			.load(emote_set_id)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		{
			let emotes = load_emote_set(&global, std::mem::take(&mut emote_set.emotes), &session).await?;
			let user_virtual_set = EmoteSetModel::from_db(emote_set, emotes, None);
			connection_model.emote_set = Some(user_virtual_set);
		}
	};

	connection_model.user = Some(UserModel::from_db(
		user,
		None,
		None,
		emote_sets,
		editors,
		&global.config.api.cdn_origin,
	));

	Ok(Json(connection_model))
}

#[utoipa::path(
    delete,
    path = "/v3/users/{id}",
    tag = "users",
    responses(
        (status = 204, description = "User Deleted"),
        (status = 404, description = "User Not Found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "The ID of the user"),
    ),
)]
#[tracing::instrument]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.delete.go#L33
pub async fn delete_user_by_id() -> Result<impl IntoResponse, ApiError> {
	// will be left unimplemented because it is unused
	Ok(ApiError::NOT_IMPLEMENTED)
}

// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.update-connection.go#L86
#[derive(Debug, Clone, Deserialize)]
struct UpdateUserConnectionBody {
	#[allow(unused)]
	new_user_id: UserId,
}

#[utoipa::path(
    patch,
    path = "/v3/users/{id}/connections/{connection_id}",
    tag = "users",
    responses(
        (status = 200, description = "User Connection", body = UserConnectionModel),
        (status = 404, description = "User Connection Not Found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "The ID of the user"),
        ("connection_id" = String, Path, description = "The ID of the connection"),
    ),
)]
#[tracing::instrument(skip_all, fields(id = %id, connection_id = %connection_id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.update-connection.go#L34
pub async fn update_user_connection_by_id(
	State(global): State<Arc<Global>>,
	Path((id, connection_id)): Path<(UserId, String)>,
	Json(body): Json<UpdateUserConnectionBody>,
) -> Result<impl IntoResponse, ApiError> {
	let _ = (global, body);
	// TODO: implement
	Ok(ApiError::NOT_IMPLEMENTED)
}
