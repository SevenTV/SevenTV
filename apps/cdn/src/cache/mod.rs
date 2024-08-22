use std::convert::Infallible;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

// use key::CacheKey;
use scuffle_foundations::http::server::axum::http::header::{self, HeaderMap};
use scuffle_foundations::http::server::axum::http::{HeaderValue, Response, StatusCode};
use scuffle_foundations::http::server::stream::{Body, IntoResponse};
use scuffle_foundations::telemetry::metrics::metrics;
use tokio::sync::OnceCell;

use crate::config;
use crate::global::Global;

pub mod key;

const ONE_WEEK: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 7);

type CacheKey = String;

pub struct Cache {
	inner: moka::future::Cache<CacheKey, CachedResponse>,
	inflight: Arc<scc::HashMap<CacheKey, Arc<Inflight>>>,
	s3_client: aws_sdk_s3::client::Client,
	request_limiter: Arc<tokio::sync::Semaphore>,
}

#[metrics]
mod cache {
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::counter::Counter;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::gauge::Gauge;

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum State {
		Hit,
		ReboundHit,
		Coalesced,
		Miss,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
	pub enum ResponseStatus {
		Success,
		NotFound,
		Timeout,
		InternalServerError,
	}

	pub fn cache_action(state: State) -> Counter;
	pub fn upstream_response(status: ResponseStatus) -> Counter;
	pub fn inflight() -> Gauge;

	pub struct InflightDropGuard;

	impl Drop for InflightDropGuard {
		fn drop(&mut self) {
			inflight().dec();
		}
	}

	impl InflightDropGuard {
		pub fn new() -> Self {
			inflight().inc();
			Self
		}
	}
}

impl Cache {
	pub fn new(config: &config::Cdn) -> Self {
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
				.expire_after(CacheExpiry)
				.weigher(|k, v: &CachedResponse| {
					u32::try_from(v.data.len() + k.len() + std::mem::size_of_val(v) + std::mem::size_of_val(k))
						.unwrap_or(u32::MAX)
				})
				.max_capacity(config.cache_capacity)
				.build(),
			inflight: Arc::new(scc::HashMap::new()),
			s3_client,
			request_limiter,
		}
	}

	pub fn entries(&self) -> u64 {
		self.inner.entry_count()
	}

	pub fn size(&self) -> u64 {
		self.inner.weighted_size()
	}

	pub fn inflight(&self) -> u64 {
		self.inflight.len() as u64
	}

	pub async fn handle_request(&self, global: &Arc<Global>, key: CacheKey) -> CachedResponse {
		if let Some(hit) = self.inner.get(&key).await {
			hit.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
			cache::cache_action(cache::State::Hit).inc();

			// return cached response
			return hit;
		}

		let mut insert = false;

		let entry = Arc::clone(&global.cache.inflight.entry_async(key.clone()).await.or_insert_with(|| {
			insert = true;

			Arc::new(Inflight {
				token: tokio_util::sync::CancellationToken::new(),
				response: OnceCell::new(),
			})
		}));

		if !insert {
			tracing::debug!(key = %key, "pending");
			cache::cache_action(cache::State::Coalesced).inc();
			// pending
			entry.token.cancelled().await;
			return entry.response.get().cloned().unwrap_or_else(CachedResponse::general_error);
		}

		struct PanicDropGuard(Option<(CacheKey, Arc<Inflight>, Arc<Global>)>);

		impl PanicDropGuard {
			fn new(key: CacheKey, entry: Arc<Inflight>, global: Arc<Global>) -> Self {
				Self(Some((key, entry, global)))
			}

			async fn disarm(mut self) {
				let Some((key, entry, global)) = self.0.take() else {
					return;
				};

				entry.token.cancel();
				global.cache.inflight.remove_async(&key).await;
			}

			fn entry(&self) -> &Arc<Inflight> {
				&self.0.as_ref().unwrap().1
			}

			fn global(&self) -> &Arc<Global> {
				&self.0.as_ref().unwrap().2
			}

			fn key(&self) -> &CacheKey {
				&self.0.as_ref().unwrap().0
			}
		}

		impl Drop for PanicDropGuard {
			fn drop(&mut self) {
				let Some((key, entry, global)) = self.0.take() else {
					return;
				};

				entry.token.cancel();
				global.cache.inflight.remove(&key);
			}
		}

		let guard = PanicDropGuard::new(key, entry, Arc::clone(&global));

		if let Some(cached) = self.inner.get(guard.key()).await {
			tracing::debug!(key = %guard.key(), "rebounded hit");
			cache::cache_action(cache::State::ReboundHit).inc();
			guard.entry().response.set(cached.clone()).expect("unreachable");
			guard.disarm().await;
			return cached.clone();
		}

		cache::cache_action(cache::State::Miss).inc();

		let cached = tokio::spawn(async move {
			// request file
			let cached = guard.global().cache.request_key(guard.global(), guard.key()).await;

			guard.entry().response.set(cached.clone()).expect("unreachable");

			if !cached.max_age.is_zero() {
				guard.global().cache.inner.insert(guard.key().clone(), cached.clone()).await;
				tracing::debug!(key = %guard.key(), "cached");
			}

			guard.disarm().await;

			cached
		});

		cached.await.unwrap_or_else(|e| {
			tracing::error!(error = %e, "task failed");
			CachedResponse::general_error()
		})
	}

	async fn do_req(&self, global: &Arc<Global>, key: &CacheKey) -> Result<CachedResponse, S3ErrorWrapper> {
		let _inflight = cache::InflightDropGuard::new();
		let _permit = self.request_limiter.acquire().await.expect("semaphore closed");

		tracing::debug!(key = %key, "requesting origin");

		tokio::time::timeout(
			std::time::Duration::from_secs(global.config.cdn.origin_request_timeout),
			async {
				Ok(CachedResponse::from_s3_response(
					self.s3_client
						.get_object()
						.bucket(&global.config.cdn.bucket.name)
						.key(key.to_string())
						.send()
						.await?,
				)
				.await?)
			},
		)
		.await?
	}

	async fn request_key(&self, global: &Arc<Global>, key: &CacheKey) -> CachedResponse {
		match self.do_req(global, key).await {
			Ok(response) => {
				cache::upstream_response(cache::ResponseStatus::Success).inc();
				response
			}
			Err(S3ErrorWrapper::SDK(aws_sdk_s3::error::SdkError::ServiceError(e))) if e.err().is_no_such_key() => {
				cache::upstream_response(cache::ResponseStatus::NotFound).inc();
				CachedResponse::not_found()
			}
			Err(S3ErrorWrapper::Timeout(_)) => {
				tracing::error!(key = %key, "timeout while requesting cdn file");
				cache::upstream_response(cache::ResponseStatus::Timeout).inc();
				CachedResponse::timeout()
			}
			Err(e) => {
				tracing::error!(key = %key, error = %e, "failed to request cdn file");
				cache::upstream_response(cache::ResponseStatus::InternalServerError).inc();
				CachedResponse::general_error()
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
enum S3ErrorWrapper {
	#[error("sdk error: {0}")]
	SDK(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>),
	#[error("timeout")]
	Timeout(#[from] tokio::time::error::Elapsed),
	#[error("bytes error: {0}")]
	Bytes(#[from] aws_sdk_s3::primitives::ByteStreamError),
}

/// Safe to clone
#[derive(Debug, Clone)]
pub struct Inflight {
	/// This token is pending as long as the request to the origin is pending.
	/// "Cancellation" is an unfortunate name for this because it is not used to
	/// cancel anything but rather notify everyone waiting that the cache is
	/// ready to be queried.
	token: tokio_util::sync::CancellationToken,
	/// The response once it is ready
	response: OnceCell<CachedResponse>,
}

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
			max_age: std::time::Duration::from_secs(10),
			hits: Arc::new(AtomicUsize::new(0)),
		}
	}

	pub fn timeout() -> Self {
		Self {
			data: CachedData::InternalServerError,
			date: chrono::Utc::now(),
			max_age: std::time::Duration::ZERO,
			hits: Arc::new(AtomicUsize::new(0)),
		}
	}

	pub fn general_error() -> Self {
		Self {
			data: CachedData::InternalServerError,
			date: chrono::Utc::now(),
			max_age: std::time::Duration::ZERO,
			hits: Arc::new(AtomicUsize::new(0)),
		}
	}
}

#[derive(Debug, Clone)]
pub enum CachedData {
	Bytes {
		content_type: Option<String>,
		chunks: Box<[bytes::Bytes]>,
	},
	NotFound,
	InternalServerError,
}

impl CachedData {
	pub fn len(&self) -> usize {
		match self {
			Self::Bytes { chunks, .. } => chunks.iter().map(|c| c.len()).sum(),
			Self::NotFound => 0,
			Self::InternalServerError => 0,
		}
	}
}

impl IntoResponse for CachedData {
	fn into_response(self) -> scuffle_foundations::http::server::stream::Response {
		match self {
			Self::Bytes { chunks, content_type } => {
				let mut headers = HeaderMap::new();

				if let Some(content_type) = content_type.as_deref().and_then(|c| c.try_into().ok()) {
					headers.insert(header::CONTENT_TYPE, content_type);
				}

				headers.insert(
					header::CONTENT_LENGTH,
					chunks.iter().map(|c| c.len()).sum::<usize>().to_string().try_into().unwrap(),
				);

				(
					headers,
					Body::from_stream(futures::stream::iter(chunks.to_vec().into_iter().map(Ok::<_, Infallible>))),
				)
					.into_response()
			}
			Self::NotFound => StatusCode::NOT_FOUND.into_response(),
			Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
		}
	}
}

impl IntoResponse for CachedResponse {
	fn into_response(self) -> Response<Body> {
		let mut data = self.data.into_response();

		if self.max_age.as_secs() == 0 {
			data.headers_mut()
				.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
		} else {
			let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);

			let age = chrono::Utc::now() - self.date;
			data.headers_mut()
				.insert("x-7tv-cache-hits", hits.to_string().try_into().unwrap());
			data.headers_mut().insert(
				"x-7tv-cache",
				if hits == 0 {
					HeaderValue::from_static("miss")
				} else {
					HeaderValue::from_static("hit")
				},
			);

			data.headers_mut()
				.insert(header::AGE, age.num_seconds().to_string().try_into().unwrap());
			data.headers_mut().insert(
				header::CACHE_CONTROL,
				format!("public, max-age={}, immutable", self.max_age.as_secs())
					.try_into()
					.unwrap(),
			);
		}

		data
	}
}

impl CachedResponse {
	pub async fn from_s3_response(
		mut value: aws_sdk_s3::operation::get_object::GetObjectOutput,
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
			.unwrap_or(ONE_WEEK);

		let mut chunks = Vec::new();

		while let Some(chunk) = value.body.next().await.transpose()? {
			chunks.push(chunk);
		}

		Ok(Self {
			data: CachedData::Bytes {
				chunks: chunks.into_boxed_slice(),
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
