use std::sync::Arc;

use scuffle_foundations::http::server::axum::extract::{Path, State};
use scuffle_foundations::http::server::axum::routing::get;
use scuffle_foundations::http::server::axum::{Json, Router};
use scuffle_foundations::telemetry::metrics::metrics;

// use shared::database::badge::BadgeId;
// use shared::database::emote::EmoteId;
// use shared::database::user::UserId;

// use crate::cache::key::{CacheKey, ImageFile};
use crate::cache::CachedResponse;
use crate::global::Global;

#[metrics]
mod http {
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::counter::Counter;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::gauge::Gauge;

	pub struct CacheInflightDropGuard(());

	impl CacheInflightDropGuard {
		pub fn new() -> Self {
			request_count().inc();
			inflight().inc();
			Self(())
		}
	}

	impl Drop for CacheInflightDropGuard {
		fn drop(&mut self) {
			inflight().dec();
		}
	}

	pub fn inflight() -> Gauge;
	pub fn request_count() -> Counter;
}

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", get(root)).route("/*key", get(any))
	// .route("/badge/:id/:file", get(badge))
	// .route("/emote/:id/:file", get(emote))
	// .route("/user/:user/:avatar_id/:file", get(user_profile_picture))
	// .route("/misc/*key", get(misc))
	// .route("/JUICERS.png", get(juicers))
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
		remaining: global.config.cdn.cache_capacity as i64 - global.cache.size() as i64,
		inflight: global.cache.inflight(),
	})
}

async fn any(Path(key): Path<String>, State(global): State<Arc<Global>>) -> CachedResponse {
	let _guard = http::CacheInflightDropGuard::new();
	global.cache.handle_request(&global, key).await
}

// async fn badge(Path((id, file)): Path<(BadgeId, ImageFile)>, State(global):
// State<Arc<Global>>) -> CachedResponse { 	global.cache.handle_request(&global,
// CacheKey::Badge { id, file }).await }

// async fn emote(Path((id, file)): Path<(EmoteId, ImageFile)>, State(global):
// State<Arc<Global>>) -> CachedResponse { 	global.cache.handle_request(&global,
// CacheKey::Emote { id, file }).await }

// async fn user_profile_picture(
// 	Path((user, avatar_id, file)): Path<(UserId, String, ImageFile)>,
// 	State(global): State<Arc<Global>>,
// ) -> CachedResponse {
// 	global
// 		.cache
// 		.handle_request(&global, CacheKey::UserProfilePicture { user, avatar_id,
// file }) 		.await
// }

// async fn misc(Path(key): Path<String>, State(global): State<Arc<Global>>) ->
// CachedResponse { 	global.cache.handle_request(&global, CacheKey::Misc { key
// }).await }

// async fn juicers(State(global): State<Arc<Global>>) -> CachedResponse {
// 	global.cache.handle_request(&global, CacheKey::Juicers).await
// }
