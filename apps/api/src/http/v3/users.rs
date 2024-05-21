use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, put};
use axum::{Json, Router};
use hyper::StatusCode;
use shared::database::{UserConnectionId, UserId};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;

#[derive(utoipa::OpenApi)]
#[openapi(
	paths(
		get_user_by_id,
		upload_user_profile_picture,
		get_user_presences_by_platform,
		get_user_by_platform_user_id,
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
		.route("/:id/presences", get(get_user_presences_by_platform))
		.route("/:platform/{platform_id}", get(get_user_by_platform_user_id))
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

	Ok(Json(
		user.into_old_model(
			user_connections,
			None,
			None,
			emote_sets
				.into_iter()
				.map(|emote_set| emote_set.into_old_model_partial(None))
				.collect(),
			editors.into_iter().filter_map(|editor| editor.into_old_model()).collect(),
			&global.config().api.cdn_base_url,
		),
	))
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
#[tracing::instrument(skip_all, fields(id = %id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.pictures.go#L61
pub async fn upload_user_profile_picture(
	State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError> {
	let _ = global;
	Ok(ApiError::NOT_IMPLEMENTED)
}

#[utoipa::path(
    get,
    path = "/v3/users/{id}/presences",
    tag = "users",
    responses(
        (status = 200, description = "User Presences", body = Vec<UserModel>),
    ),
    params(
        ("id" = String, Path, description = "The ID of the user"),
    ),
)]
#[tracing::instrument(skip_all, fields(id = %id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.presence.write.go#L41
pub async fn get_user_presences_by_platform(
	State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError> {
	let _ = global;
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
pub async fn get_user_by_platform_user_id(
	State(global): State<Arc<Global>>,
	Path((platform, platform_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
	let _ = global;
	Ok(ApiError::NOT_IMPLEMENTED)
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
#[tracing::instrument(skip_all, fields(id = %id))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/users/users.delete.go#L33
pub async fn delete_user_by_id(
	State(global): State<Arc<Global>>,
	Path(id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError> {
	let _ = global;
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
