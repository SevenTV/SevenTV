use std::sync::{atomic::AtomicUsize, Arc};

use scuffle_foundations::http::server::{
	axum::http::{
		header::{self, HeaderMap},
		HeaderValue, Response, StatusCode,
	},
	stream::{Body, IntoResponse},
};

use crate::config;

const ONE_DAY: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);

pub struct Cache {
	inner: moka::future::Cache<CacheKey, CachedResponse>,
	path_meta: Arc<scc::HashMap<String, PathMeta>>,
	s3_bucket_name: String,
	s3_client: aws_sdk_s3::client::Client,
	request_limiter: Arc<tokio::sync::Semaphore>,
	request_timeout: std::time::Duration,
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
	#[error("failed to fetch origin: {0}")]
	S3(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>),
	#[error("s3 byte stream error: {0}")]
	S3ByteStream(#[from] aws_sdk_s3::primitives::ByteStreamError),
	#[error("origin request timed out")]
	Timeout,
}

impl Cache {
	pub fn new(config: &config::Cdn) -> Self {
		let path_meta = Arc::new(scc::HashMap::new());
		let pm = Arc::clone(&path_meta);

		let s3_client = {
			let mut s3_config = if let Some(endpoint) = &config.bucket.endpoint {
				aws_sdk_s3::config::Builder::new().endpoint_url(endpoint)
			} else {
				aws_sdk_s3::config::Builder::new()
			}
			.region(aws_sdk_s3::config::Region::new(config.bucket.region.clone()))
			.force_path_style(true);

			if let Some(credentials) = config.bucket.credentials.to_credentials() {
				s3_config = s3_config.credentials_provider(credentials);
			}

			let config = s3_config.build();

			aws_sdk_s3::Client::from_conf(config)
		};

		let request_limiter = Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_requests as usize));

		Self {
			inner: moka::future::Cache::builder()
				.async_eviction_listener(move |key: Arc<CacheKey>, _, _| {
					let path_meta = Arc::clone(&pm);

					Box::pin(async move {
						tracing::debug!(key = %key, "evicting cache entry");
						path_meta.remove_async(key.as_ref()).await;
					})
				})
				.expire_after(CacheExpiry)
				.weigher(|_, v: &CachedResponse| v.data.len() as u32)
				.max_capacity(config.cache_capacity)
				.build(),
			path_meta,
			s3_bucket_name: config.bucket.name.clone(),
			s3_client,
			request_limiter,
			request_timeout: std::time::Duration::from_secs(config.origin_request_timeout),
		}
	}

	pub async fn handle_request(&self, key: String) -> Result<CachedResponse, CacheError> {
		'fetch: loop {
			let meta = self.path_meta.read_async(&key, |_, meta| meta.clone()).await;

			if let Some(meta) = meta {
				match meta {
					PathMeta::Pending { coalescing } => {
						// wait for the request to finish
						coalescing.cancelled().await;

						// refetch the meta, it should be PathMeta::Cached now
						continue 'fetch;
					}
					PathMeta::Cached => {
						if let Some(hit) = self.inner.get(&key).await {
							hit.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

							// return cached response
							return Ok(hit);
						} else {
							// something is wrong
							unreachable!("cache is missing a cached response");
						}
					}
				}
			} else {
				// miss
				break 'fetch;
			}
		}

		// miss

		let coalescing = tokio_util::sync::CancellationToken::new();

		self.path_meta
			.insert_async(
				key.clone(),
				PathMeta::Pending {
					coalescing: coalescing.clone(),
				},
			)
			.await
			.map_err(|_| ())
			.expect("unreachable");

		// request file
		let cached = match self.request_key(key.clone()).await {
			Ok(response) => response,
			Err(e) => {
				// remove the pending entry
				self.path_meta.remove_async(&key).await;
				coalescing.cancel();
				return Err(e);
			}
		};

		self.path_meta.update_async(&key.clone(), |_, e| *e = PathMeta::Cached).await;

		self.inner.insert(key, cached.clone()).await;

		// tell others waiting that the cache is ready
		coalescing.cancel();

		Ok(cached)
	}

	async fn request_key(&self, key: String) -> Result<CachedResponse, CacheError> {
		// request file
		let response = {
			// we are never closing the semaphore, so we can expect it to be open here, right? Clueless
			let _permit = self.request_limiter.acquire().await.expect("semaphore closed");

			tokio::time::timeout(
				self.request_timeout,
				self.s3_client.get_object().bucket(&self.s3_bucket_name).key(&key).send(),
			)
			.await
			.map_err(|_| CacheError::Timeout)?
		};

		match response {
			Ok(response) => Ok(CachedResponse::from_s3_response(response).await?),
			Err(aws_sdk_s3::error::SdkError::ServiceError(e)) if e.err().is_no_such_key() => Ok(CachedResponse::not_found()),
			Err(e) => Err(e.into()),
		}
	}
}

/// Safe to clone
#[derive(Debug, Clone)]
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
	pub data: CachedData,
	pub date: chrono::DateTime<chrono::Utc>,
	pub max_age: std::time::Duration,
	pub hits: Arc<AtomicUsize>,
}

impl CachedResponse {
	pub fn not_found() -> Self {
		Self {
			data: CachedData::NotFound,
			date: chrono::Utc::now(),
			max_age: ONE_DAY,
			hits: Arc::new(AtomicUsize::new(0)),
		}
	}
}

#[derive(Debug, Clone)]
pub enum CachedData {
	Bytes {
		data: bytes::Bytes,
		content_type: Option<String>,
	},
	NotFound,
}

impl CachedData {
	pub fn len(&self) -> usize {
		match self {
			Self::Bytes { data, .. } => data.len(),
			Self::NotFound => 0,
		}
	}
}

impl IntoResponse for CachedData {
	fn into_response(self) -> scuffle_foundations::http::server::stream::Response {
		match self {
			Self::Bytes { data, content_type } => {
				let mut headers = HeaderMap::new();

				if let Some(content_type) = content_type.as_deref().and_then(|c| c.try_into().ok()) {
					headers.insert(header::CONTENT_TYPE, content_type);
				}

				(headers, data).into_response()
			},
			Self::NotFound => {
				StatusCode::NOT_FOUND.into_response()
			},
		}
	}
}

impl IntoResponse for CachedResponse {
	fn into_response(self) -> Response<Body> {
		let mut data = self.data.into_response();

		let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);

		let age = chrono::Utc::now() - self.date;

		data.headers_mut().insert("x-7tv-cache-hits", hits.to_string().try_into().unwrap());
		data.headers_mut().insert(
			"x-7tv-cache",
			if hits == 0 {
				HeaderValue::from_static("miss")
			} else {
				HeaderValue::from_static("hit")
			},
		);
		data.headers_mut().insert(header::AGE, age.num_seconds().to_string().try_into().unwrap());

		data
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
			data: CachedData::Bytes {
				data: value.body.collect().await?.into_bytes(),
				content_type: value.content_type,
			},
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
