use std::{str::FromStr, sync::Arc};

use scc::ebr::Guard;
use scuffle_foundations::http::server::axum::{
	extract::{Path, State},
	http::{header, HeaderMap, HeaderName, StatusCode},
	routing::any,
	Router,
};

use crate::{
	cache::{CacheKey, CachedResponse, PathMeta},
	global::Global,
};

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/", any(root)).route("/*key", any(cdn_route))
}

async fn root() -> &'static str {
	"Welcome to the 7TV CDN!"
}

async fn cdn_route(
	mut headers: HeaderMap,
	Path(key): Path<String>,
	State(global): State<Arc<Global>>,
) -> Result<CachedResponse, StatusCode> {
	let meta = global.path_meta.peek(&key, &Guard::new()).map(Arc::clone);

	if let Some(meta) = meta {
		let important_headers = meta
			.vary_headers
			.iter()
			.cloned()
			.filter_map(|h| headers.remove(&h).map(|v| (h, v)))
			.collect();

		let cache_key = CacheKey {
			path: key.clone(),
			important_headers,
		};

		if let Some(hit) = global.cache.get(&cache_key).await {
			return Ok(hit);
		}
	}

	// miss

	// request file
	let response = global
		.http_client
		.get(format!("{}/{}", global.config.cdn.bucket_url, key))
		.timeout(std::time::Duration::from_secs(5))
		.send()
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to get cdn file");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	if response
		.headers()
		.get(header::VARY)
		.is_some_and(|v| v.to_str().is_ok_and(|v| v == "*"))
	{
		// uncacheable
		return Ok(CachedResponse::from_reqwest(response).await.map_err(|e| {
			tracing::error!(error = %e, "failed to read cdn file");
			StatusCode::INTERNAL_SERVER_ERROR
		})?);
	}

	let vary_headers: Vec<_> = response
		.headers()
		.get(header::VARY)
		.and_then(|v| v.to_str().ok())
		.map(|v| v.split(',').filter_map(|v| HeaderName::from_str(v.trim()).ok()).collect())
		.unwrap_or_default();

	let cached = CachedResponse::from_reqwest(response).await.map_err(|e| {
		tracing::error!(error = %e, "failed to read cdn file");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let meta = Arc::new(PathMeta { vary_headers });

	global.path_meta.insert(key.clone(), Arc::clone(&meta)).map_err(|_| {
		tracing::error!("failed to insert cdn file meta");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let important_headers = meta
		.vary_headers
		.iter()
		.cloned()
		.filter_map(|h| headers.remove(&h).map(|v| (h, v)))
		.collect();

	let cache_key = CacheKey {
		path: key,
		important_headers,
	};

	global.cache.insert(cache_key, cached.clone()).await;

	Ok(cached)
}
