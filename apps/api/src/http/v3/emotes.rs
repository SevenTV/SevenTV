use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;

use crate::{global::Global, http::error::ApiError};

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .get("/", get_emotes)
        .post("/", create_emote)
        .get("/{emote_id}", get_emote_by_id)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Emote {}

#[utoipa::path(
    get,
    path = "/v3/emotes",
    responses(
        (status = 200, description = "Emotes", body = Vec<Emote>),
    ),
)]
pub async fn get_emotes(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEmoteData {}

#[utoipa::path(
    post,
    path = "/v3/emotes",
    request_body = Bytes,
    responses(
        (status = 201, description = "Emote Created"),
    ),
    params(
        ("X-Emote-Data" = XEmoteData, Header, description = "The properties of the emote"),
    ),
)]
pub async fn create_emote(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/emotes/{emote_id}",
    responses(
        (status = 200, description = "Emote", body = Emote),
        // (status = 404, description = "Emote Not Found", body = ApiError)
    ),
    params(
        ("emote_id" = String, Path, description = "The ID of the emote"),
    ),
)]
pub async fn get_emote_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}