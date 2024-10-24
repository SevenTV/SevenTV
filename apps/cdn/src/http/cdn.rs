use std::sync::Arc;

use http::Uri;
use scuffle_foundations::http::server::axum::extract::{Path, State};
use scuffle_foundations::http::server::axum::routing::get;
use scuffle_foundations::http::server::axum::{Json, Router};
use shared::cdn::key::{CacheKey, ImageFile};
use shared::database::badge::BadgeId;
use shared::database::emote::EmoteId;
use shared::database::paint::{PaintId, PaintLayerId};
use shared::database::user::profile_picture::UserProfilePictureId;
use shared::database::user::UserId;

use crate::cache::CachedResponse;
use crate::global::Global;

pub fn routes(_: &Arc<Global>) -> Router<Arc<Global>> {
	Router::new()
		.route("/", get(root))
		.route("/badge/:id/:file", get(badge))
		.route("/emote/:id/:file", get(emote))
		.route("/user/:user/profile-picture/:avatar_id/:file", get(user_profile_picture))
		.route("/paint/:id/layer/:layer/:file", get(paint_layer))
}

#[derive(Debug, serde::Serialize)]
struct Welcome {
	message: String,
	name: String,
	pod_name: String,
	node_name: String,
	size: u64,
	entries: u64,
	remaining: i64,
	inflight: u64,
}

fn redirect_to_new_url(key: CacheKey) -> CachedResponse {
	CachedResponse::redirect(format!("/{key}"))
}

async fn root(State(global): State<Arc<Global>>) -> Json<Welcome> {
	Json(Welcome {
		message: "Welcome to the 7TV CDN!".to_string(),
		name: global.config.cdn.server_name.clone(),
		pod_name: global.config.pod.name.clone(),
		node_name: global.config.pod.node_name.clone(),
		size: global.cache.size(),
		entries: global.cache.entries(),
		remaining: global.cache.capacity() as i64 - global.cache.size() as i64,
		inflight: global.cache.inflight(),
	})
}

async fn badge(
	Path((badge_id, file)): Path<(BadgeId, ImageFile)>,
	State(global): State<Arc<Global>>,
	uri: Uri,
) -> CachedResponse {
	let key = CacheKey::Badge { badge_id, file };
	if uri.path().trim_start_matches('/') != key.to_string() {
		return redirect_to_new_url(key);
	}

	global.cache.handle_request(&global, key).await
}

async fn emote(
	Path((emote_id, file)): Path<(EmoteId, ImageFile)>,
	State(global): State<Arc<Global>>,
	uri: Uri,
) -> CachedResponse {
	let key = CacheKey::Emote { emote_id, file };
	if uri.path().trim_start_matches('/') != key.to_string() {
		return redirect_to_new_url(key);
	}

	global.cache.handle_request(&global, key).await
}

async fn user_profile_picture(
	Path((user_id, avatar_id, file)): Path<(UserId, UserProfilePictureId, ImageFile)>,
	State(global): State<Arc<Global>>,
	uri: Uri,
) -> CachedResponse {
	let key = CacheKey::UserProfilePicture {
		user_id,
		avatar_id,
		file,
	};
	if uri.path().trim_start_matches('/') != key.to_string() {
		return redirect_to_new_url(key);
	}

	global.cache.handle_request(&global, key).await
}

async fn paint_layer(
	Path((paint_id, layer_id, file)): Path<(PaintId, PaintLayerId, ImageFile)>,
	State(global): State<Arc<Global>>,
	uri: Uri,
) -> CachedResponse {
	let key = CacheKey::Paint {
		paint_id,
		layer_id,
		file,
	};
	if uri.path().trim_start_matches('/') != key.to_string() {
		return redirect_to_new_url(key);
	}

	global.cache.handle_request(&global, key).await
}
