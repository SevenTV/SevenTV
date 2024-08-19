use std::sync::{atomic::AtomicUsize, Arc};

use scuffle_foundations::http::server::{
	axum::http::{
		header::{self, HeaderMap},
		HeaderValue, Response, StatusCode,
	},
	stream::{Body, IntoResponse},
};
use shared::database::{badge::BadgeId, emote::EmoteId, user::UserId, Id};
use tokio::sync::OnceCell;

use crate::{config, global::Global};

const ONE_DAY: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24);

pub struct Cache {
	inner: moka::future::Cache<CacheKey, CachedResponse>,
	inflight: Arc<scc::HashMap<CacheKey, Inflight>>,
	s3_client: aws_sdk_s3::client::Client,
	request_limiter: Arc<tokio::sync::Semaphore>,
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
	#[error("failed to fetch origin: {0}")]
	S3(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>),
	#[error("s3 byte stream error: {0}")]
	S3ByteStream(#[from] aws_sdk_s3::primitives::ByteStreamError),
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

	pub async fn handle_request(&self, global: &Arc<Global>, key: CacheKey) -> Result<CachedResponse, CacheError> {
		if let Some(hit) = self.inner.get(&key).await {
			hit.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

			// return cached response
			return Ok(hit);
		}

		let mut insert = false;

		let entry = self.inflight.entry_async(key.clone()).await.or_insert_with(|| {
			insert = true;

			Inflight {
				token: tokio_util::sync::CancellationToken::new(),
				response: Arc::new(OnceCell::new()),
			}
		});

		if !insert {
			// pending
			entry.token.cancelled().await;
			return Ok(entry.response.get().cloned().expect("unreachable"));
		}

		// miss

		let _guard = entry.token.clone().drop_guard();

		// request file
		let cached = self.request_key(global, key.clone()).await?;

		entry.response.set(cached.clone()).expect("unreachable");

		self.inflight.remove_async(&key).await;

		if !cached.max_age.is_zero() {
			self.inner.insert(key, cached.clone()).await;
		}

		Ok(cached)
	}

	async fn request_key(&self, global: &Arc<Global>, key: CacheKey) -> Result<CachedResponse, CacheError> {
		// request file
		let response = {
			// we are never closing the semaphore, so we can expect it to be open here, right? Clueless
			let _permit = self.request_limiter.acquire().await.expect("semaphore closed");

			tokio::time::timeout(
				std::time::Duration::from_secs(global.config.cdn.origin_request_timeout),
				self.s3_client
					.get_object()
					.bucket(&global.config.cdn.bucket.name)
					.key(key.get_path(global.config.cdn.migration_timestamp))
					.send(),
			)
			.await
		};

		match response {
			Ok(Ok(response)) => Ok(CachedResponse::from_s3_response(response).await?),
			Ok(Err(aws_sdk_s3::error::SdkError::ServiceError(e))) if e.err().is_no_such_key() => {
				Ok(CachedResponse::not_found())
			}
			Ok(Err(e)) => Err(e.into()),
			Err(_) => Ok(CachedResponse::timeout()),
		}
	}
}

/// Safe to clone
#[derive(Debug, Clone)]
pub struct Inflight {
	/// This token is pending as long as the request to the origin is pending.
	/// "Cancellation" is an unfortunate name for this because it is not used to cancel anything but rather notify everyone waiting that the cache is ready to be queried.
	token: tokio_util::sync::CancellationToken,
	/// The response once it is ready
	response: Arc<OnceCell<CachedResponse>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
	Badge { id: BadgeId, file: String },
	Emote { id: EmoteId, file: String },
	UserProfilePicture { user: UserId, avatar_id: String, file: String },
	Misc { key: String },
	Juicers,
}

fn legacy_id<S>(id: Id<S>, migration_timestamp: chrono::DateTime<chrono::Utc>) -> String {
	// When the requested id is older than the migration timestamp, we need to convert it back to an old object id
	(id.timestamp() < migration_timestamp)
		.then_some(id.as_object_id().map(|i| i.to_string()))
		.flatten()
		.unwrap_or(id.to_string())
}

impl CacheKey {
	pub fn get_path(&self, migration_timestamp: chrono::DateTime<chrono::Utc>) -> String {
		match self {
			Self::Badge { id, file } => format!("badge/{}/{file}", legacy_id(*id, migration_timestamp)),
			Self::Emote { id, file } => format!("emote/{}/{file}", legacy_id(*id, migration_timestamp)),
			Self::UserProfilePicture { user, avatar_id, file } => {
				format!("user/{}/{avatar_id}/{file}", legacy_id(*user, migration_timestamp))
			}
			Self::Misc { key } => format!("misc/{key}"),
			Self::Juicers => "JUICERS.png".to_string(),
		}
	}
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
			data: CachedData::NotFound,
			date: chrono::Utc::now(),
			max_age: std::time::Duration::from_secs(0),
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
			}
			Self::NotFound => StatusCode::NOT_FOUND.into_response(),
		}
	}
}

impl IntoResponse for CachedResponse {
	fn into_response(self) -> Response<Body> {
		let mut data = self.data.into_response();

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
