use std::collections::HashMap;
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::StatusCode;
use scuffle_utils::http::ext::{OptionExt, ResultExt};
use scuffle_utils::http::router::builder::RouterBuilder;
use scuffle_utils::http::router::ext::RequestExt;
use scuffle_utils::http::router::Router;
use scuffle_utils::http::RouteError;
use shared::database::FeaturePermission;
use shared::http::{json_response, Body};
use shared::types::old::EmoteSetModel;
use utoipa::OpenApi;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::RequestGlobalExt;

#[derive(OpenApi)]
#[openapi(paths(get_emote_set_by_id), components(schemas(EmoteSetModel)))]
pub struct Docs;

pub fn routes(_: &Arc<Global>) -> RouterBuilder<Incoming, Body, RouteError<ApiError>> {
	Router::builder().get("/emote-sets/{id}", get_emote_set_by_id)
}

#[utoipa::path(
    get,
    path = "/v3/emote-sets/{id}",
    tag = "emote-sets",
    responses(
        (status = 200, description = "Emote Set", body = EmoteSetModel, content_type = "application/json"),
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
		.and_then(|id| id.parse().ok())
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

	let emotes = global
		.emote_by_id_loader()
		.load_many(emote_set_emotes.iter().map(|emote| emote.emote_id))
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query emotes"))?;

	let users = global
		.user_by_id_loader()
		.load_many(
			&global,
			emotes.values().filter_map(|emote| emote.owner_id).chain(emote_set.owner_id),
		)
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query users"))?;

	let global_config = global
		.global_config_loader()
		.load(())
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query global config"))?
		.ok_or((StatusCode::INTERNAL_SERVER_ERROR, "global config not found"))?;

	let roles = {
		let mut roles = global
			.role_by_id_loader()
			.load_many(users.values().flat_map(|user| user.entitled_cache.role_ids.iter().copied()))
			.await
			.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query roles"))?;

		global_config
			.role_ids
			.iter()
			.filter_map(|id| roles.remove(id))
			.collect::<Vec<_>>()
	};

	let users = users
		.into_iter()
		.map(|(id, user)| {
			let permissions = user.compute_permissions(&roles);
			(id, (user, permissions))
		})
		.collect::<HashMap<_, _>>();

	let file_sets = global
		.file_set_by_id_loader()
		.load_many(
			users
				.values()
				.filter_map(|(user, permissions)| {
					if permissions.has(FeaturePermission::UseCustomProfilePicture) {
						user.style.active_profile_picture_id
					} else {
						None
					}
				})
				.chain(emotes.values().map(|emote| emote.file_set_id)),
		)
		.await
		.map_ignore_err_route((StatusCode::INTERNAL_SERVER_ERROR, "failed to query file sets"))?;

	let cdn_base_url = &global.config().api.cdn_base_url;

	let users = users
		.into_values()
		.map(|(user, _)| {
			let profile_picture_file_set = user.style.active_profile_picture_id.and_then(|id| file_sets.get(&id));

			// This api doesnt seem to return the user's badges, paints and connections so
			// we can ignore them.
			user.into_old_model_partial(Vec::new(), profile_picture_file_set, None, None, cdn_base_url)
		})
		.map(|user| (user.id, user))
		.collect::<HashMap<_, _>>();

	let emotes = emotes
		.into_iter()
		.filter_map(|(id, emote)| {
			let owner = emote.owner_id.and_then(|id| users.get(&id)).cloned();
			let file_set = file_sets.get(&emote.file_set_id)?;

			Some((
				id,
				emote.into_old_model_partial(owner, file_set, &global.config().api.cdn_base_url),
			))
		})
		.collect::<HashMap<_, _>>();

	let owner = emote_set.owner_id.and_then(|id| users.get(&id)).cloned();

	json_response(emote_set.into_old_model(
		emote_set_emotes.into_iter().map(|emote| {
			let partial = emotes.get(&emote.emote_id).cloned();
			(emote, partial)
		}),
		owner,
	))
}
