use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::ext::RequestExt;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::Body;
use shared::id::parse_id;
use shared::types::old::UserPartialModel;
use ulid::Ulid;
use utoipa::OpenApi;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::RequestGlobalExt;

#[derive(OpenApi)]
#[openapi(paths(get_emote_set_by_id), components(schemas(EmoteSet)))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder().get("/emote-sets/{id}", get_emote_set_by_id)
}

#[utoipa::path(
    get,
    path = "/v3/emote-sets/{id}",
    tag = "emote-sets",
    responses(
        (status = 200, description = "Emote Set", body = EmoteSet, content_type = "application/json"),
        // (status = 404, description = "Emote Set Not Found", body = ApiError)
    ),
    params(
        ("id" = String, Path, description = "The ID of the emote set"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emote-sets/emote-sets.by-id.go#L42
pub async fn get_emote_set_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;

	let id = req
		.param("id")
		.and_then(parse_id)
		.map_err_route((StatusCode::BAD_REQUEST, "invalid emote set ID"))?;

	let emote_set = global
		.emote_set_by_id_loader()
		.load(id)
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query emote sets"))?
		.map_err_route((StatusCode::NOT_FOUND, "emote set not found"))?;

	let emote_set_emotes = global
		.emote_set_emote_by_id_loader()
		.load(id)
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query emote set emotes"))?
		.unwrap_or_default();

	Err((StatusCode::NOT_IMPLEMENTED, "not implemented").into())
}
