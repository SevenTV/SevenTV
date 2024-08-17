use std::sync::Arc;

use scuffle_foundations::http::server::axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	routing::any,
	Router,
};

use crate::{cache::CachedResponse, global::Global};

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", any(root)).route("/*key", any(cdn_route))
}

async fn root() -> &'static str {
	"Welcome to the 7TV CDN!"
}

async fn cdn_route(
	headers: HeaderMap,
	Path(key): Path<String>,
	State(global): State<Arc<Global>>,
) -> Result<CachedResponse, StatusCode> {
	global.cache.handle_request(&global, key, headers).await.map_err(|e| {
		tracing::error!(error = %e, "failed to handle cdn request");
		StatusCode::INTERNAL_SERVER_ERROR
	})
}
