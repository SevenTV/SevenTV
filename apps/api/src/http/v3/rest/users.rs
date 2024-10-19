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
use shared::database::emote::EmoteFlags;
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{FlagPermission, PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::connection::Platform;
use shared::database::user::editor::{EditorUserPermission, UserEditorId};
use shared::database::user::profile_picture::{UserProfilePicture, UserProfilePictureId};
use shared::database::user::{User, UserId, UserStyle};
use shared::database::{Id, MongoCollection};
use shared::event::{
	EventUserPresencePlatform, InternalEvent, InternalEventData, InternalEventPayload, InternalEventUserPresenceData,
	InternalEventUserPresenceDataEmoteSet,
};
use shared::old_types::{
	EmoteSetModel, EmoteSetPartialModel, UserConnectionModel, UserConnectionPartialModel, UserEditorModel, UserModel,
};

use super::types::{PresenceKind, PresenceModel, UserPresencePlatform, UserPresenceWriteRequest};
use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
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
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

	if user.has(FlagPermission::Hidden) && Some(user.id) != session.user_id() && !session.has(UserPermission::ViewHidden) {
		return Err(ApiError::not_found(ApiErrorCode::LoadError, "user not found"));
	}

	let emote_sets = global
		.emote_set_by_user_id_loader
		.load(user.id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
		.unwrap_or_default();

	let editors = global
		.user_editor_by_user_id_loader
		.load(user.id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
		.unwrap_or_default();

	let active_emote_set = if let Some(emote_set_id) = user.style.active_emote_set_id {
		global
			.emote_set_by_id_loader
			.load(emote_set_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
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
	let authed_user = session.user()?;

	let other_user = match id {
		TargetUser::Me => None,
		TargetUser::Other(id) => Some(
			global
				.user_loader
				.load(&global, id)
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
				.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?,
		),
	};

	let target_user = other_user.as_ref().unwrap_or(authed_user);

	if !target_user.computed.permissions.has(UserPermission::UseCustomProfilePicture) {
		return Err(ApiError::forbidden(
			ApiErrorCode::LackingPrivileges,
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
				if err.code == image_processor::ErrorCode::Decode as i32
					|| err.code == image_processor::ErrorCode::InvalidInput as i32
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
							active_profile_picture: Some(profile_picture_id),
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
	State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
	Extension(session): Extension<Session>,
	Json(presence): Json<UserPresenceWriteRequest>,
) -> Result<impl IntoResponse, ApiError> {
	let req = RateLimitRequest::new(RateLimitResource::UserPresenceWrite, &session);

	req.http(&global, async {
		if presence.kind != PresenceKind::Channel {
			return Err(ApiError::not_implemented(
				ApiErrorCode::BadRequest,
				"only channel presence (1) is supported",
			));
		}

		if presence.data.id.is_empty() {
			return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "missing data.id"));
		} else if presence.data.id.len() > 100 || !presence.data.id.is_ascii() {
			return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "data.id is invalid"));
		}

		let Some(user) = global
			.user_loader
			.load(&global, id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		else {
			return Err(ApiError::not_found(ApiErrorCode::LoadError, "user not found"));
		};

		let active_badge = if let Some(id) = user
			.style
			.active_badge_id
			.and_then(|id| user.computed.entitlements.badges.contains(&id).then_some(id))
		{
			global
				.badge_by_id_loader
				.load(id)
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load badge"))?
		} else {
			None
		};

		let active_paint = if let Some(id) = user
			.style
			.active_paint_id
			.and_then(|id| user.computed.entitlements.paints.contains(&id).then_some(id))
		{
			global
				.paint_by_id_loader
				.load(id)
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paint"))?
		} else {
			None
		};

		let personal_emote_sets = global
			.emote_set_by_id_loader
			.load_many(
				user.computed.entitlements.emote_sets.iter().copied().chain(
					user.style
						.personal_emote_set_id
						.and_then(|id| user.has(UserPermission::UsePersonalEmoteSet).then_some(id)),
				),
			)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.into_values()
			.collect::<Vec<_>>();

		let emotes = global
			.emote_by_id_loader
			.load_many_merged(
				personal_emote_sets
					.iter()
					.flat_map(|s| s.emotes.iter().map(|e| e.id))
					.collect::<Vec<_>>(),
			)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?;

		let mut sets = vec![];

		for set in personal_emote_sets {
			let emotes = set
				.emotes
				.iter()
				.filter_map(|e| emotes.get(e.id))
				.filter(|e| e.flags.contains(EmoteFlags::ApprovedPersonal))
				.cloned()
				.collect();

			sets.push(InternalEventUserPresenceDataEmoteSet { emote_set: set, emotes });
		}

		let payload = rmp_serde::to_vec_named(&InternalEventPayload::new(Some(InternalEvent {
			actor: None,
			session_id: None,
			data: InternalEventData::UserPresence(Box::new(InternalEventUserPresenceData {
				user,
				platform: match presence.data.platform {
					UserPresencePlatform::Twitch => {
						EventUserPresencePlatform::Twitch(presence.data.id.parse().map_err(|_| {
							ApiError::bad_request(ApiErrorCode::BadRequest, "data.id is not a valid twitch id")
						})?)
					}
					UserPresencePlatform::Kick => {
						EventUserPresencePlatform::Kick(presence.data.id.parse().map_err(|_| {
							ApiError::bad_request(ApiErrorCode::BadRequest, "data.id is not a valid kick id")
						})?)
					}
					UserPresencePlatform::Youtube => EventUserPresencePlatform::Youtube(presence.data.id),
				},
				active_badge,
				active_paint,
				personal_emote_sets: sets,
			})),
			timestamp: chrono::Utc::now(),
		})))
		.map_err(|err| {
			tracing::error!(error = %err, "failed to serialize event");
			ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to serialize event")
		})?;

		global.nats.publish("api.v4.events", payload.into()).await.map_err(|err| {
			tracing::error!(error = %err, "failed to publish event");
			ApiError::internal_server_error(ApiErrorCode::Unknown, "failed to publish event")
		})?;

		let now = chrono::Utc::now();

		Ok(Json(PresenceModel {
			id: Id::nil(),
			user_id: id,
			timestamp: now.timestamp_millis(),
			ttl: now.timestamp_millis(),
			kind: PresenceKind::Channel,
		}))
	})
	.await
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
	let platform = Platform::from_str(&platform.to_lowercase())
		.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid platform"))?;

	let user = global
		.user_loader
		.load_user(
			&global,
			global
				.user_by_platform_id_loader
				.load((platform, platform_id.clone()))
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
				.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?,
		)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

	if user.has(FlagPermission::Hidden)
		&& Some(user.id) != session.user_id()
		&& !session.permissions().has(UserPermission::ViewHidden)
	{
		return Err(ApiError::not_found(ApiErrorCode::LoadError, "user not found"));
	}

	let connection = user
		.connections
		.iter()
		.find(|c| c.platform == platform && c.platform_id == platform_id)
		.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

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
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
		.unwrap_or_default()
		.into_iter()
		.filter_map(UserEditorModel::from_db)
		.collect::<Vec<_>>();

	// query user emote sets
	let emote_sets = global
		.emote_set_by_user_id_loader
		.load(user.id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
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
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
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
	Ok(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
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
	Path((id, connection_id)): Path<(UserId, String)>,
	Json(_body): Json<UpdateUserConnectionBody>,
) -> Result<impl IntoResponse, ApiError> {
	// won't be implemented
	Ok(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
}
