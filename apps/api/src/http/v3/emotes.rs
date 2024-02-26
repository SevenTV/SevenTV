use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;

use crate::{global::Global, http::error::ApiError};

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .post("/", create_emote)
        .get("/{id}", get_emote_by_id)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Emote {}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEmoteData {}

#[utoipa::path(
    post,
    path = "/v3/emotes",
    request_body(content = &[u8], description = "Image Binary Data", content_type = "image/png, image/jpeg, image/gif, image/webp, image/avif"),
    responses(
        (status = 201, description = "Emote Created"),
    ),
    params(
        ("X-Emote-Data" = XEmoteData, Header, description = "The properties of the emote"),
    ),
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.create.go#L58
pub async fn create_emote(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/emotes/{id}",
    responses(
        (status = 200, description = "Emote", body = Emote),
        // (status = 404, description = "Emote Not Found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "The ID of the emote"),
    ),
)]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.by-id.go#L36
pub async fn get_emote_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}
