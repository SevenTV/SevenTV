use std::hash::Hash;

use moka::future::Cache;
use scuffle_foundations::http::server::{
	axum::http::header::{self, HeaderMap, HeaderName},
	axum::http::Response,
	stream::{Body, IntoResponse},
};

const ONE_DAY: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);

pub struct PathMeta {
	pub vary_headers: Vec<HeaderName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
	pub path: String,
	pub important_headers: Vec<(header::HeaderName, header::HeaderValue)>,
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
	pub headers: HeaderMap,
	pub body: bytes::Bytes,
	pub max_age: std::time::Duration,
}

impl IntoResponse for CachedResponse {
	fn into_response(self) -> Response<Body> {
		(self.headers, self.body).into_response()
	}
}

impl CachedResponse {
	pub async fn from_reqwest(res: reqwest::Response) -> reqwest::Result<Self> {
		let cache_control = res
			.headers()
			.get(header::CACHE_CONTROL)
			.and_then(|v| v.to_str().ok())
			.map(|c| c.to_ascii_lowercase());

		let max_age = cache_control
			.as_deref()
			.and_then(|c| c.split(',').find_map(|v| v.strip_prefix("max-age=")))
			.and_then(|v| v.trim().parse::<u64>().ok())
			.map(std::time::Duration::from_secs)
			.unwrap_or(ONE_DAY);

		Ok(Self {
			headers: res.headers().clone(),
			body: res.bytes().await?,
			max_age,
		})
	}
}

struct CacheExpiry;

impl moka::Expiry<CacheKey, CachedResponse> for CacheExpiry {
	fn expire_after_create(
		&self,
		_key: &CacheKey,
		value: &CachedResponse,
		_created_at: std::time::Instant,
	) -> Option<std::time::Duration> {
		Some(value.max_age)
	}
}

pub fn create() -> Cache<CacheKey, CachedResponse> {
	Cache::builder()
		.expire_after(CacheExpiry)
		.weigher(|_, v: &CachedResponse| v.body.len() as u32)
		.build()
}
