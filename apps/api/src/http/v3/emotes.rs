use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::ext::RequestExt;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::http::{json_response, Body};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::RequestGlobalExt;

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_emote, get_emote_by_id), components(schemas(XEmoteData)))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder().post("/", create_emote).get("/{id}", get_emote_by_id)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct XEmoteData {}

#[utoipa::path(
    post,
    path = "/v3/emotes",
    tag = "emotes",
    // Currently utoipa does not support multiple request body types so we use `image/*` as a placeholder
    // See https://github.com/juhaku/utoipa/pull/876
    request_body(content = &[u8], description = "Image Binary Data", content_type = "image/*"),
    responses(
        (status = 201, description = "Emote Created"),
    ),
    params(
        ("X-Emote-Data" = XEmoteData, Header, description = "The properties of the emote"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.create.go#L58
pub async fn create_emote(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	todo!()
}

#[utoipa::path(
    get,
    path = "/v3/emotes/{id}",
    tag = "emotes",
    responses(
        (status = 200, description = "Emote", body = EmoteModel),
        (status = 404, description = "Emote Not Found")
    ),
    params(
        ("id" = String, Path, description = "The ID of the emote"),
    ),
)]
#[tracing::instrument(level = "info", skip(req), fields(path = %req.uri().path(), method = %req.method()))]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/emotes/emotes.by-id.go#L36
pub async fn get_emote_by_id(req: hyper::Request<Incoming>) -> Result<hyper::Response<Body>, RouteError<ApiError>> {
	let global: Arc<Global> = req.get_global()?;

	let id = req.param("id").map_err_route((StatusCode::BAD_REQUEST, "missing id"))?;

	let id = id.parse().map_ignore_err_route((StatusCode::BAD_REQUEST, "invalid id"))?;

	let Some(emote) = global
		.emote_by_id_loader()
		.load(id)
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to load emote"))?
	else {
		return Err((StatusCode::NOT_FOUND, "emote not found").into());
	};

	let Some(file_set) = global
		.file_set_by_id_loader()
		.load(emote.file_set_id)
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to load file set"))?
	else {
		return Err((StatusCode::INTERNAL_SERVER_ERROR, "emote file set not found").into());
	};

	let owner = match emote.owner_id {
		Some(owner) => {
			let user = global
				.user_by_id_loader()
				.load(&global, owner)
				.await
				.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to load user"))?;

			user.map(|u| u.into_old_model_partial(todo!(), todo!(), todo!(), todo!(), &global.config().api.cdn_base_url))
		}
		None => None,
	};

	json_response(emote.into_old_model(owner, &file_set, &global.config().api.cdn_base_url))
}
