use std::sync::Arc;

use hyper::body::Incoming;
use scuffle_utils::http::{router::{builder::RouterBuilder, Router}, RouteError};
use shared::http::Body;

use crate::{global::Global, http::error::ApiError};

pub fn routes(global: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
    Router::builder()
        .get("/emote-sets", get_emote_sets)
        .get("/emote-sets/{emote_set_id}", get_emote_set_by_id)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EmoteSet {}

#[utoipa::path(
    get,
    path = "/v3/emote-sets",
    responses(
        (status = 200, description = "Emote Sets", body = Vec<EmoteSet>),
    ),
)]
pub async fn get_emote_sets(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}

#[utoipa::path(
    get,
    path = "/v3/emote-sets/{emote_set_id}",
    responses(
        (status = 200, description = "Emote Set", body = EmoteSet),
        // (status = 404, description = "Emote Set Not Found", body = ApiError)
    ),
    params(
        ("emote_set_id" = String, Path, description = "The ID of the emote set"),
    ),
)]
pub async fn get_emote_set_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
    todo!()
}
