use std::str::FromStr;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::{Extension, Json, Router};
use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use scuffle_image_processor_proto::{self as image_processor, ProcessImageResponse, ProcessImageResponseUploadInfo};
use serde::Deserialize;
use shared::database::{
	Collection, FeaturePermission, ImageSet, ImageSetInput, Platform, User, UserConnection, UserConnectionId, UserId,
	UserPermission,
};
use shared::types::old::{PresenceModel, UserConnectionModel, UserConnectionPartialModel};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::emote_set_loader::{fake_user_set, get_fake_set_for_user_active_sets};

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
) -> Result<impl IntoResponse, ApiError> {
	let user = global
		.user_by_id_loader()
		.load(&global, id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "user not found"))?;

	let emote_sets = global
		.emote_set_by_user_id_loader()
		.load(user.id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default();

	let user_connections = global
		.user_connection_by_user_id_loader()
		.load(user.id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default();

	let editors = global
		.user_editor_by_user_id_loader()
		.load(user.id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default();

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

	// the fake user emote set
	let fake_user_set =
		fake_user_set(user.id, permissions.emote_set_slots_limit.unwrap_or(600)).into_old_model(vec![], None);

	let mut old_model = user.into_old_model(
		user_connections,
		None,
		None,
		emote_sets
			.into_iter()
			.map(|emote_set| emote_set.into_old_model_partial(None))
			.collect(),
		editors.into_iter().filter_map(|editor| editor.into_old_model()).collect(),
		&global.config().api.cdn_base_url,
	);

	old_model.connections.iter_mut().for_each(|conn| {
		conn.emote_set_id = Some(fake_user_set.id);
		conn.emote_set = Some(fake_user_set.clone());
	});

	Ok(Json(old_model))
}

#[derive(Debug, Clone, Deserialize)]
enum TargetUser {
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
	auth_session: Option<Extension<AuthSession>>,
	body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
	let authed_user_id = auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id();

	let id = match id {
		TargetUser::Me => authed_user_id,
		TargetUser::Other(id) => id,
	};

	let user = global
		.user_by_id_loader()
		.load(&global, id)
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::NOT_FOUND)?;

	let authed_user = global
		.user_by_id_loader()
		.load(&global, authed_user_id)
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
			.load_many(authed_user.entitled_cache.role_ids.iter().copied())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		global_config
			.role_ids
			.iter()
			.filter_map(|id| roles.remove(id))
			.collect::<Vec<_>>()
	};

	let permissions = authed_user.compute_permissions(&roles);

	if authed_user_id == id {
		// When someone wants to change their own profile picture
		if !permissions.has(FeaturePermission::UseCustomProfilePicture) {
			return Err(ApiError::FORBIDDEN);
		}
	} else {
		// When someone wants to change another user's profile picture, they must have `UserPermission::Edit`
		if !permissions.has(UserPermission::Edit) {
			return Err(ApiError::FORBIDDEN);
		}
	}

	// check if user already has a pending profile picture change
	if let Some(ImageSet {
		input: ImageSetInput::Pending { .. },
		..
	}) = user.style.active_profile_picture
	{
		return Err(ApiError::new_const(
			StatusCode::CONFLICT,
			"profile picture change already pending",
		));
	}

	let input = match global.image_processor().upload_profile_picture(id, body).await {
		Ok(ProcessImageResponse {
			error: None,
			upload_info: Some(ProcessImageResponseUploadInfo {
				path: Some(path),
				content_type,
				size,
			}),
			..
		}) => ImageSetInput::Pending {
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

	let image_set = ImageSet { input, outputs: vec![] };
	let image_set = to_bson(&image_set).map_err(|e| {
		tracing::error!(error = %e, "failed to serialize image set");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	User::collection(global.db())
		.update_one(
			doc! {
				"_id": user.id,
			},
			doc! {
				"$set": {
					"style.active_profile_picture": image_set,
				}
			},
			None,
		)
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "failed to update user");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	Ok(StatusCode::OK)
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
) -> Result<impl IntoResponse, ApiError> {
	let platform = Platform::from_str(&platform.to_lowercase()).map_err(|_| ApiError::BAD_REQUEST)?;
	let platform = to_bson(&platform).map_err(|e| {
		tracing::error!(error = %e, "failed to serialize platform");
		ApiError::INTERNAL_SERVER_ERROR
	})?;

	let connection = UserConnection::collection(global.db())
		.find_one(doc! { "platform": platform, "platform_id": platform_id }, None)
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to find user connection");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.ok_or(ApiError::NOT_FOUND)?;

	// query the user
	let user = global
		.user_by_id_loader()
		.load(&global, connection.user_id)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	let mut connection_model: UserConnectionModel = UserConnectionPartialModel::from(connection).into();

	// query user
	// query all user connections
	let connections = global
		.user_connection_by_user_id_loader()
		.load(user.id)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default();

	let editors = global
		.user_editor_by_user_id_loader()
		.load(user.id)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default()
		.into_iter()
		.filter_map(|e| e.into_old_model())
		.collect::<Vec<_>>();

	// query user emote sets
	let emote_sets = global
		.emote_set_by_user_id_loader()
		.load(user.id)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.unwrap_or_default()
		.into_iter()
		.map(|s| s.into_old_model_partial(None))
		.collect::<Vec<_>>();

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

	let user_fake_set = get_fake_set_for_user_active_sets(
		&global,
		user.clone(),
		connections.clone(),
		permissions.emote_set_slots_limit.unwrap_or(600),
	)
	.await?;
	connection_model.emote_set_id = Some(user_fake_set.id);
	connection_model.emote_set = Some(user_fake_set);

	let user_full = user.into_old_model(
		connections,
		None,
		None,
		emote_sets,
		editors,
		&global.config().api.cdn_base_url,
	);

	connection_model.user = Some(user_full);

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
	Path((id, connection_id)): Path<(UserId, UserConnectionId)>,
) -> Result<impl IntoResponse, ApiError> {
	let _ = global;
	Ok(ApiError::NOT_IMPLEMENTED)
}