use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use key::CacheKey;
use scc::hash_map::OccupiedEntry;
use scuffle_foundations::http::server::axum::http::header::{self, HeaderMap};
use scuffle_foundations::http::server::axum::http::{HeaderValue, Response, StatusCode};
use scuffle_foundations::http::server::stream::{Body, IntoResponse};
use tokio::sync::OnceCell;

use crate::config;
use crate::global::Global;

pub mod key;

const ONE_WEEK: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 7);

pub struct Cache {
	inner: moka::future::Cache<CacheKey, CachedResponse>,
	inflight: Arc<scc::HashMap<CacheKey, Inflight>>,
	s3_client: aws_sdk_s3::client::Client,
	request_limiter: Arc<tokio::sync::Semaphore>,
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
				.weigher(|_, v: &CachedResponse| v.data.len() as u32)
				.max_capacity(config.cache_capacity)
				.build(),
			inflight: Arc::new(scc::HashMap::new()),
			s3_client,
			request_limiter,
		}
	}

	pub async fn handle_request(&self, global: &Arc<Global>, key: CacheKey) -> CachedResponse {
		if let Some(hit) = self.inner.get(&key).await {
			hit.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

			// return cached response
			return hit;
		}

		let global = Arc::clone(&global);

		let cached = tokio::spawn(async move {
			let mut insert = false;

			let entry = global.cache.inflight.entry_async(key.clone()).await.or_insert_with(|| {
				insert = true;

				Inflight {
					token: tokio_util::sync::CancellationToken::new(),
					response: OnceCell::new(),
				}
			});

			struct EntryDropGuard<'a>(Option<OccupiedEntry<'a, CacheKey, Inflight>>);

			impl<'a> EntryDropGuard<'a> {
				fn new(entry: OccupiedEntry<'a, CacheKey, Inflight>) -> Self {
					Self(Some(entry))
				}

				fn entry(&self) -> &OccupiedEntry<'a, CacheKey, Inflight> {
					self.0.as_ref().unwrap()
				}
			}

			impl Drop for EntryDropGuard<'_> {
				fn drop(&mut self) {
					let _ = self.0.take().unwrap().remove();
				}
			}

			let entry = EntryDropGuard::new(entry);

			if !insert {
				// pending
				entry.entry().token.cancelled().await;
				return entry
					.entry()
					.response
					.get()
					.cloned()
					.unwrap_or_else(CachedResponse::general_error);
			}

			let _guard = entry.entry().token.clone().drop_guard();

			// request file
			let cached = global.cache.request_key(&global, key.clone()).await;

			entry.entry().response.set(cached.clone()).expect("unreachable");

			if !cached.max_age.is_zero() {
				global.cache.inner.insert(key, cached.clone()).await;
			}

			cached
		});

		cached.await.unwrap_or_else(|e| {
			tracing::error!(error = %e, "task failed");
			CachedResponse::general_error()
		})
	}

	async fn request_key(&self, global: &Arc<Global>, key: CacheKey) -> CachedResponse {
		// request file
		let response = {
			// we are never closing the semaphore, so we can expect it to be open here,
			// right? Clueless
			let _permit = self.request_limiter.acquire().await.expect("semaphore closed");

			tracing::debug!(key = %key, "requesting origin");

			tokio::time::timeout(
				std::time::Duration::from_secs(global.config.cdn.origin_request_timeout),
				self.s3_client
					.get_object()
					.bucket(&global.config.cdn.bucket.name)
					.key(key.to_string())
					.send(),
			)
			.await
		};

		match response {
			Ok(Ok(response)) => match CachedResponse::from_s3_response(response).await {
				Ok(response) => response,
				Err(e) => {
					tracing::error!(key = %key, error = %e, "failed to parse cdn file");
					CachedResponse::general_error()
				}
			},
			Ok(Err(aws_sdk_s3::error::SdkError::ServiceError(e))) if e.err().is_no_such_key() => CachedResponse::not_found(),
			Ok(Err(e)) => {
				let e = S3ErrorWrapper(e);
				tracing::error!(key = %key, error = %e, "failed to request cdn file");
				CachedResponse::general_error()
			}
			Err(_) => {
				tracing::error!(key = %key, "timeout while requesting cdn file");
				CachedResponse::timeout()
			}
		}
	}
}

struct S3ErrorWrapper(aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>);

impl std::fmt::Display for S3ErrorWrapper {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.0 {
			aws_sdk_s3::error::SdkError::ConstructionFailure(_) => write!(f, "{}", self.0),
			aws_sdk_s3::error::SdkError::TimeoutError(_) => write!(f, "{}", self.0),
			aws_sdk_s3::error::SdkError::DispatchFailure(_) => write!(f, "{}", self.0),
			aws_sdk_s3::error::SdkError::ResponseError(_) => write!(f, "{}", self.0),
			aws_sdk_s3::error::SdkError::ServiceError(e) => {
				let e = e.err();
				match e {
					aws_sdk_s3::operation::get_object::GetObjectError::InvalidObjectState(e) => write!(f, "{}", e),
					aws_sdk_s3::operation::get_object::GetObjectError::NoSuchKey(e) => write!(f, "{}", e),
					_ => write!(f, "{}", self.0),
				}
			}
			_ => write!(f, "{}", self.0),
		}
	}
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
		data: bytes::Bytes,
		content_type: Option<String>,
	},
	NotFound,
	InternalServerError,
}

impl CachedData {
	pub fn len(&self) -> usize {
		match self {
			Self::Bytes { data, .. } => data.len(),
			Self::NotFound => 0,
			Self::InternalServerError => 0,
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
			.unwrap_or(ONE_WEEK);

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
