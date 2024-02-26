use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;

use crate::{global::Global, http::error::ApiError};

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .get("/", get_users)
        .get("/{user_id}", get_user_by_id) 
        .get("/{user_id}/profile-picture", get_user_profile_picture_by_id)
        .get("/{platform}/presences", get_user_presences_by_platform)
        .get("/{platform}/{platform_user_id}", get_user_by_platform_user_id)
        .delete("/{user_id}", delete_user_by_id)
        .patch("/{user_id}/connections/{connection_id}", update_user_connection_by_id)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct User {}

#[utoipa::path(
    get,
    path = "/v3/users",
    responses(
        (status = 200, description = "Users", body = Vec<User>),
    ),
)]
pub async fn get_users(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/users/{user_id}",
    responses(
        (status = 200, description = "User", body = User),
        // (status = 404, description = "User Not Found", body = ApiError)
    ),
    params(
        ("user_id" = String, Path, description = "The ID of the user"),
    ),
)]
pub async fn get_user_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/users/{user_id}/profile-picture",
    responses(
        (status = 200, description = "User Profile Picture", body = Bytes),
        // (status = 404, description = "User Profile Picture Not Found", body = ApiError)
    ),
    params(
        ("user_id" = String, Path, description = "The ID of the user"),
    ),
)]
pub async fn get_user_profile_picture_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/users/{platform}/presences",
    responses(
        (status = 200, description = "User Presences", body = Vec<User>),
    ),
    params(
        ("platform" = String, Path, description = "The platform"),
    ),
)]
pub async fn get_user_presences_by_platform(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/users/{platform}/{platform_user_id}",
    responses(
        (status = 200, description = "User", body = User),
        // (status = 404, description = "User Not Found", body = ApiError)
    ),
    params(
        ("platform" = String, Path, description = "The platform"),
        ("platform_user_id" = String, Path, description = "The ID of the user on the platform"),
    ),
)]
pub async fn get_user_by_platform_user_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    delete,
    path = "/v3/users/{user_id}",
    responses(
        (status = 204, description = "User Deleted"),
        // (status = 404, description = "User Not Found", body = ApiError)
    ),
    params(
        ("user_id" = String, Path, description = "The ID of the user"),
    ),
)]
pub async fn delete_user_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UserConnection {}

#[utoipa::path(
    patch,
    path = "/v3/users/{user_id}/connections/{connection_id}",
    responses(
        (status = 200, description = "User Connection", body = UserConnection),
        // (status = 404, description = "User Connection Not Found", body = ApiError)
    ),
    params(
        ("user_id" = String, Path, description = "The ID of the user"),
        ("connection_id" = String, Path, description = "The ID of the connection"),
    ),
)]
pub async fn update_user_connection_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}
