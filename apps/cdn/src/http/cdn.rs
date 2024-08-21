use std::sync::Arc;

use scuffle_foundations::http::server::axum::extract::{Path, State};
use scuffle_foundations::http::server::axum::routing::get;
use scuffle_foundations::http::server::axum::Router;
use shared::database::badge::BadgeId;
use shared::database::emote::EmoteId;
use shared::database::user::UserId;

use crate::cache::key::{CacheKey, ImageFile};
use crate::cache::CachedResponse;
use crate::global::Global;

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/", get(root))
		.route("/badge/:id/:file", get(badge))
		.route("/emote/:id/:file", get(emote))
		.route("/user/:user/:avatar_id/:file", get(user_profile_picture))
		.route("/misc/*key", get(misc))
		.route("/JUICERS.png", get(juicers))
}

async fn root() -> &'static str {
	"Welcome to the 7TV CDN!"
}

async fn badge(Path((id, file)): Path<(BadgeId, ImageFile)>, State(global): State<Arc<Global>>) -> CachedResponse {
	global.cache.handle_request(&global, CacheKey::Badge { id, file }).await
}

async fn emote(Path((id, file)): Path<(EmoteId, ImageFile)>, State(global): State<Arc<Global>>) -> CachedResponse {
	global.cache.handle_request(&global, CacheKey::Emote { id, file }).await
}

async fn user_profile_picture(
	Path((user, avatar_id, file)): Path<(UserId, String, ImageFile)>,
	State(global): State<Arc<Global>>,
) -> CachedResponse {
	global
		.cache
		.handle_request(&global, CacheKey::UserProfilePicture { user, avatar_id, file })
		.await
}

async fn misc(Path(key): Path<String>, State(global): State<Arc<Global>>) -> CachedResponse {
	global.cache.handle_request(&global, CacheKey::Misc { key }).await
}

async fn juicers(State(global): State<Arc<Global>>) -> CachedResponse {
	global.cache.handle_request(&global, CacheKey::Juicers).await
}
