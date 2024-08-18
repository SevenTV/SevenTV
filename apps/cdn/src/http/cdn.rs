use std::sync::Arc;

use scuffle_foundations::http::server::axum::{
	extract::{Path, State},
	http::StatusCode,
	routing::{any, get},
	Router,
};
use shared::database::emote::EmoteId;

use crate::{cache::CachedResponse, global::Global};

pub fn routes() -> Router<Arc<Global>> {
	Router::new()
		.route("/", any(root))
		.route("/emote/:id/:file", get(emote))
		.route("/*key", any(cdn_route))
}

async fn root() -> &'static str {
	"Welcome to the 7TV CDN!"
}

async fn cdn_route(Path(key): Path<String>, State(global): State<Arc<Global>>) -> Result<CachedResponse, StatusCode> {
	global.cache.handle_request(key).await.map_err(|e| {
		tracing::error!(error = %e, "failed to handle cdn request");
		StatusCode::INTERNAL_SERVER_ERROR
	})
}

async fn emote(
	Path((id, file)): Path<(EmoteId, String)>,
	State(global): State<Arc<Global>>,
) -> Result<CachedResponse, StatusCode> {
	// When the requested id is older than the migration timestamp, we need to convert it back to an old object id
	let id = (id.timestamp() < global.config.cdn.migration_timestamp)
		.then_some(id.as_object_id().map(|i| i.to_string()))
		.flatten()
		.unwrap_or(id.to_string());

	global.cache.handle_request(format!("emote/{id}/{file}")).await.map_err(|e| {
		tracing::error!(error = %e, "failed to handle cdn request");
		StatusCode::INTERNAL_SERVER_ERROR
	})
}
