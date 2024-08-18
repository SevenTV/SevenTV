use std::sync::{atomic::AtomicUsize, Arc};

use scc::{ebr::Guard, TreeIndex};
use scuffle_foundations::http::server::{
	axum::http::{
		header::{self, HeaderMap},
		HeaderValue, Response,
	},
	stream::{Body, IntoResponse},
};

use crate::config::S3BucketConfig;

const ONE_DAY: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);

pub struct Cache {
	inner: moka::future::Cache<CacheKey, CachedResponse>,
	path_meta: Arc<TreeIndex<String, Arc<PathMeta>>>,
	s3_bucket_name: String,
	s3_client: aws_sdk_s3::client::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
	#[error("failed to fetch origin: {0}")]
	S3(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>),
	#[error("s3 byte stream error: {0}")]
	S3ByteStream(#[from] aws_sdk_s3::primitives::ByteStreamError),
	#[error("failed to update path meta index")]
	PathMetaUpdate,
}

impl Cache {
	pub fn new(capacity: u64, bucket_config: &S3BucketConfig) -> Self {
		let path_meta = Arc::new(TreeIndex::new());
		let pm = Arc::clone(&path_meta);

		let mut config = if let Some(endpoint) = &bucket_config.endpoint {
			aws_sdk_s3::config::Builder::new().endpoint_url(endpoint)
		} else {
			aws_sdk_s3::config::Builder::new()
		}
		.region(aws_sdk_s3::config::Region::new(bucket_config.region.clone()))
		.force_path_style(true);

		if let Some(credentials) = bucket_config.credentials.to_credentials() {
			config = config.credentials_provider(credentials);
		}

		let config = config.build();

		let s3_client = aws_sdk_s3::Client::from_conf(config);

		Self {
			inner: moka::future::Cache::builder()
				.async_eviction_listener(move |key: Arc<CacheKey>, _, _| {
					let path_meta = Arc::clone(&pm);

					Box::pin(async move {
						path_meta.remove_async(key.as_ref()).await;
					})
				})
				.expire_after(CacheExpiry)
				.weigher(|_, v: &CachedResponse| v.data.len() as u32)
				.max_capacity(capacity)
				.build(),
			path_meta,
			s3_bucket_name: bucket_config.name.clone(),
			s3_client,
		}
	}

	pub async fn handle_request(&self, key: String) -> Result<CachedResponse, CacheError> {
		loop {
			let meta = self.path_meta.peek(&key, &Guard::new()).map(Arc::clone);

			if let Some(meta) = meta {
				match meta.as_ref() {
					PathMeta::Pending { coalescing } => {
						// wait for the request to finish
						coalescing.cancelled().await;

						// refetch the meta, it should be PathMeta::Cached now
						continue;
					}
					PathMeta::Cached => {
						if let Some(hit) = self.inner.get(&key).await {
							hit.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

							// return cached response
							return Ok(hit);
						}
					}
				}
			} else {
				// miss
				break;
			}
		}

		// miss

		let coalescing = tokio_util::sync::CancellationToken::new();

		self.path_meta
			.insert(
				key.clone(),
				Arc::new(PathMeta::Pending {
					coalescing: coalescing.clone(),
				}),
			)
			.map_err(|_| CacheError::PathMetaUpdate)?;

		// request file
		let response = self
			.s3_client
			.get_object()
			.bucket(&self.s3_bucket_name)
			.key(&key)
			.send()
			.await?;

		let cached = CachedResponse::from_s3_response(response).await?;

		self.path_meta
			.insert(key.clone(), Arc::new(PathMeta::Cached))
			.map_err(|_| CacheError::PathMetaUpdate)?;

		self.inner.insert(key, cached.clone()).await;

		// tell others waiting that the cache is ready
		coalescing.cancel();

		Ok(cached)
	}
}

pub enum PathMeta {
	Pending {
		/// This token is pending as long as the request to the origin is pending.
		/// "Cancellation" is an unfortunate name for this because it is not used to cancel anything but rather notify everyone waiting that the cache is ready to be queried.
		coalescing: tokio_util::sync::CancellationToken,
	},
	Cached,
}

pub type CacheKey = String;

#[derive(Debug, Clone)]
pub struct CachedResponse {
	pub data: bytes::Bytes,
	pub date: chrono::DateTime<chrono::Utc>,
	pub max_age: std::time::Duration,
	pub hits: Arc<AtomicUsize>,
}

impl IntoResponse for CachedResponse {
	fn into_response(self) -> Response<Body> {
		let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);

		let age = chrono::Utc::now() - self.date;

		let mut headers = HeaderMap::new();

		headers.insert("x-7tv-cache-hits", hits.to_string().try_into().unwrap());
		headers.insert(
			"x-7tv-cache",
			if hits == 0 {
				HeaderValue::from_static("miss")
			} else {
				HeaderValue::from_static("hit")
			},
		);
		headers.insert(header::AGE, age.num_seconds().to_string().try_into().unwrap());

		(headers, self.data).into_response()
	}
}

impl CachedResponse {
	pub async fn from_s3_response(
		value: aws_sdk_s3::operation::get_object::GetObjectOutput,
	) -> Result<Self, aws_sdk_s3::primitives::ByteStreamError> {
		let date = chrono::Utc::now();

		let max_age = value
			.cache_control
			.map(|c| c.to_ascii_lowercase())
			.as_deref()
			.and_then(|c| c.split(',').find_map(|v| v.strip_prefix("max-age=")))
			.and_then(|v| v.trim().parse::<u64>().ok())
			.map(std::time::Duration::from_secs)
			.or_else(|| {
				let expires = value
					.expires_string
					.and_then(|e| chrono::DateTime::parse_from_rfc2822(&e).ok());
				expires.and_then(|e| e.signed_duration_since(date).to_std().ok())
			})
			.unwrap_or(ONE_DAY);

		Ok(Self {
			data: value.body.collect().await?.into_bytes(),
			date,
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
