use std::{
	hash::Hash,
	str::FromStr,
	sync::{atomic::AtomicUsize, Arc},
};

use reqwest::header::HeaderValue;
use scc::{ebr::Guard, TreeIndex};
use scuffle_foundations::http::server::{
	axum::http::header::{self, HeaderMap, HeaderName},
	axum::http::{Response, StatusCode},
	stream::{Body, IntoResponse},
};

use crate::global::Global;

const ONE_DAY: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);

pub struct Cache {
	inner: moka::future::Cache<CacheKey, CachedResponse>,
	path_meta: Arc<TreeIndex<String, Arc<PathMeta>>>,
	http_client: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
	#[error("failed to get cdn file: {0}")]
	Reqwest(#[from] reqwest::Error),
	#[error("failed to update path meta index")]
	PathMetaUpdate,
}

impl Cache {
	pub fn new(capacity: u64) -> Self {
		let path_meta = Arc::new(TreeIndex::new());
		let pm = Arc::clone(&path_meta);

		Self {
			inner: moka::future::Cache::builder()
				.async_eviction_listener(move |key: Arc<CacheKey>, _, _| {
					let path_meta = Arc::clone(&pm);

					Box::pin(async move {
						path_meta.remove_async(&key.path).await;
					})
				})
				.expire_after(CacheExpiry)
				.weigher(|_, v: &CachedResponse| v.body.len() as u32)
				.max_capacity(capacity)
				.build(),
			path_meta,
			http_client: reqwest::Client::new(),
		}
	}

	pub async fn handle_request(
		&self,
		global: &Arc<Global>,
		key: String,
		mut headers: HeaderMap,
	) -> Result<CachedResponse, CacheError> {
		let meta = self.path_meta.peek(&key, &Guard::new()).map(Arc::clone);

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

			if let Some(hit) = self.inner.get(&cache_key).await {
				hit.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

				return Ok(hit);
			}
		}

		// miss

		// request file
		let response = self
			.http_client
			.get(format!("{}/{}", global.config.cdn.bucket_url, key))
			.timeout(std::time::Duration::from_secs(5))
			.send()
			.await?;

		if response
			.headers()
			.get(header::VARY)
			.is_some_and(|v| v.to_str().is_ok_and(|v| v == "*"))
		{
			// uncacheable
			return Ok(CachedResponse::from_reqwest(response).await?);
		}

		let vary_headers: Vec<_> = response
			.headers()
			.get(header::VARY)
			.and_then(|v| v.to_str().ok())
			.map(|v| v.split(',').filter_map(|v| HeaderName::from_str(v.trim()).ok()).collect())
			.unwrap_or_default();

		let cached = CachedResponse::from_reqwest(response).await?;

		let meta = Arc::new(PathMeta { vary_headers });

		self.path_meta
			.insert(key.clone(), Arc::clone(&meta))
			.map_err(|_| CacheError::PathMetaUpdate)?;

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

		self.inner.insert(cache_key, cached.clone()).await;

		Ok(cached)
	}
}

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
	pub status: StatusCode,
	pub date: chrono::DateTime<chrono::Utc>,
	pub headers: HeaderMap,
	pub body: bytes::Bytes,
	pub max_age: std::time::Duration,
	pub hits: Arc<AtomicUsize>,
}

impl IntoResponse for CachedResponse {
	fn into_response(mut self) -> Response<Body> {
		let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);

		let age = chrono::Utc::now() - self.date;

		self.headers.insert("x-7tv-cache-hits", hits.to_string().try_into().unwrap());
		self.headers.insert(
			"x-7tv-cache",
			if hits == 0 {
				HeaderValue::from_static("miss")
			} else {
				HeaderValue::from_static("hit")
			},
		);
		self.headers.insert("age", age.num_seconds().to_string().try_into().unwrap());

		(self.status, self.headers, self.body).into_response()
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

		let date = res
			.headers()
			.get(header::DATE)
			.and_then(|v| v.to_str().ok())
			.and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
			.map(|d| d.to_utc())
			.unwrap_or_else(chrono::Utc::now);

		Ok(Self {
			status: res.status(),
			date,
			headers: res.headers().clone(),
			body: res.bytes().await?,
			max_age,
			hits: Arc::new(AtomicUsize::new(0)),
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
