use std::sync::Arc;

use scuffle_foundations::http::server::axum::extract::{Path, State};
use scuffle_foundations::http::server::axum::routing::get;
use scuffle_foundations::http::server::axum::{Json, Router};
use shared::database::badge::BadgeId;
use shared::database::emote::EmoteId;
use shared::database::paint::{PaintId, PaintLayerId};
use shared::database::user::UserId;

use crate::cache::key::{CacheKey, ImageFile};
use crate::cache::CachedResponse;
use crate::global::Global;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/", get(root))
		.route("/badge/:id/:file", get(badge))
		.route("/emote/:id/:file", get(emote))
		.route("/user/:user/profile-picture/:avatar_id/:file", get(user_profile_picture))
		.route("/paint/:id/layer/:layer/:file", get(paint_layer))
		.route("/JUICERS.png", get(juicers))
}

#[derive(Debug, serde::Serialize)]
struct Welcome {
	message: String,
	name: String,
	size: u64,
	entries: u64,
	remaining: i64,
	inflight: u64,
}

async fn root(State(global): State<Arc<Global>>) -> Json<Welcome> {
	Json(Welcome {
		message: "Welcome to the 7TV CDN!".to_string(),
		name: global.config.cdn.server_name.clone(),
		size: global.cache.size(),
		entries: global.cache.entries(),
		remaining: global.cache.capacity() as i64 - global.cache.size() as i64,
		inflight: global.cache.inflight(),
	})
}

async fn badge(Path((badge_id, file)): Path<(BadgeId, ImageFile)>, State(global): State<Arc<Global>>) -> CachedResponse {
	global.cache.handle_request(&global, CacheKey::Badge { badge_id, file }).await
}

async fn emote(Path((emote_id, file)): Path<(EmoteId, ImageFile)>, State(global): State<Arc<Global>>) -> CachedResponse {
	global.cache.handle_request(&global, CacheKey::Emote { emote_id, file }).await
}

async fn user_profile_picture(
	Path((user_id, avatar_id, file)): Path<(UserId, String, ImageFile)>,
	State(global): State<Arc<Global>>,
) -> CachedResponse {
	global
		.cache
		.handle_request(
			&global,
			CacheKey::UserProfilePicture {
				user_id,
				avatar_id,
				file,
			},
		)
		.await
}

async fn paint_layer(
	Path((paint_id, layer_id, file)): Path<(PaintId, PaintLayerId, ImageFile)>,
	State(global): State<Arc<Global>>,
) -> CachedResponse {
	global
		.cache
		.handle_request(
			&global,
			CacheKey::Paint {
				paint_id,
				layer_id,
				file,
			},
		)
		.await
}

async fn juicers(State(global): State<Arc<Global>>) -> CachedResponse {
	global.cache.handle_request(&global, CacheKey::Juicers).await
}
