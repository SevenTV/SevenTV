use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use axum::http::HeaderName;
use axum::response::{IntoResponse, Response};
use hyper::{HeaderMap, StatusCode};
use scuffle_foundations::settings::auto_settings;
use shared::database::role::permissions::{AdminPermission, PermissionsExt};
use shared::database::user::{FullUser, UserId};

use crate::global::Global;
use crate::http::error::ApiError;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitResource {
	EmoteUpload,
	ProfilePictureUpload,
	Login,
	Search,
	UserChangeCosmetics,
	UserChangeEditor,
	UserChangeConnections,
	EmoteUpdate,
	EmoteSetCreate,
	EmoteSetChange,
	EgVaultSubscribe,
	EgVaultRedeem,
	EgVaultPaymentMethod,
}

impl std::fmt::Display for RateLimitResource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EmoteUpload => write!(f, "emote_upload"),
			Self::ProfilePictureUpload => write!(f, "profile_picture_upload"),
			Self::Login => write!(f, "login"),
			Self::Search => write!(f, "search"),
			Self::UserChangeCosmetics => write!(f, "user_change_cosmetics"),
			Self::UserChangeEditor => write!(f, "user_change_editor"),
			Self::UserChangeConnections => write!(f, "user_change_connections"),
			Self::EmoteUpdate => write!(f, "emote_update"),
			Self::EmoteSetCreate => write!(f, "emote_set_create"),
			Self::EmoteSetChange => write!(f, "emote_set_change"),
			Self::EgVaultSubscribe => write!(f, "egvault_subscribe"),
			Self::EgVaultRedeem => write!(f, "egvault_redeem"),
			Self::EgVaultPaymentMethod => write!(f, "egvault_payment_method"),
		}
	}
}

/// `RateLimiter` is a wrapper around the redis rate limiter.
///
/// Each resource has a list of buckets that are used to define the rate limit
/// for that resource.
///
/// Each bucket has a interval in seconds and a number of requests that can be
/// made in that interval.
///
/// There are different types of scopes that can be used to acquire a rate
/// limit:
/// - IP: The IP address of the user
/// - User: The user id of the user
/// - UserSession: The user session id of the user
pub struct RateLimiter {
	redis: fred::clients::RedisClient,
	ratelimit: fred::types::Function,
	limits: HashMap<RateLimitResource, LimitsConfig>,
}

#[derive(Copy)]
#[auto_settings]
#[serde(default)]
pub struct LimitsConfig {
	/// The number of seconds in the interval
	pub interval_seconds: u64,
	/// The number of requests that can be made in the interval
	pub requests: u64,
	/// If the limit is exceeded, the user will be banned for the specified
	/// amount of seconds If specified otherwise nothing will happen and the
	/// user will be allowed to make requests After their limit resets (the
	/// interval)
	pub overuse_threshold: Option<u64>,
	pub overuse_punishment: Option<u64>,
}

#[derive(Copy, Clone, Debug)]
pub enum RateLimitIdentifier {
	Ip(std::net::IpAddr),
	UserId(UserId),
}

impl From<UserId> for RateLimitIdentifier {
	fn from(id: UserId) -> Self {
		Self::UserId(id)
	}
}

impl From<std::net::IpAddr> for RateLimitIdentifier {
	fn from(ip: std::net::IpAddr) -> Self {
		Self::Ip(ip)
	}
}

impl std::fmt::Display for RateLimitIdentifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			RateLimitIdentifier::Ip(ip) => write!(f, "ip:{}", ip),
			RateLimitIdentifier::UserId(id) => write!(f, "user:{}", id),
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct RateLimitRequest {
	pub resource: RateLimitResource,
	pub id: RateLimitIdentifier,
	pub limit: Option<u64>,
	pub ticket_count: u64,
}

impl RateLimitRequest {
	pub fn new(resource: RateLimitResource, authed_user: Option<&FullUser>, ip: std::net::IpAddr) -> Self {
		Self {
			resource,
			id: authed_user
				.map(|s| RateLimitIdentifier::UserId(s.id))
				.unwrap_or(RateLimitIdentifier::Ip(ip)),
			limit: None,
			ticket_count: authed_user
				.map(|s| if s.has(AdminPermission::BypassRateLimit) { 0 } else { 1 })
				.unwrap_or(1),
		}
	}

	pub fn with_limit(mut self, limit: u64) -> Self {
		self.limit = Some(limit);
		self
	}

	pub fn with_ticket_count(mut self, ticket_count: u64) -> Self {
		self.ticket_count = ticket_count;
		self
	}
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitResponse {
	pub remaining: u64,
	pub reset: u64,
	pub limit: u64,
	pub used: u64,
	pub resource: RateLimitResource,
}

impl RateLimitResponse {
	pub const RATE_LIMIT_ERROR: ApiError = ApiError::new_const(StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded");

	pub fn header_map(&self) -> HeaderMap {
		let x_rate_limit_limit =
			HeaderName::try_from(format!("x-ratelimit-{}-limit", self.resource)).expect("invalid header name");
		let x_rate_limit_remaining =
			HeaderName::try_from(format!("x-ratelimit-{}-remaining", self.resource)).expect("invalid header name");
		let x_rate_limit_reset =
			HeaderName::try_from(format!("x-ratelimit-{}-reset", self.resource)).expect("invalid header name");
		let x_rate_limit_used =
			HeaderName::try_from(format!("x-ratelimit-{}-used", self.resource)).expect("invalid header name");

		HeaderMap::from_iter([
			(x_rate_limit_limit, self.limit.into()),
			(x_rate_limit_remaining, self.remaining.into()),
			(x_rate_limit_reset, self.reset.into()),
			(x_rate_limit_used, self.used.into()),
		])
	}

	pub fn error(&self) -> ApiError {
		Self::RATE_LIMIT_ERROR.with_extra_headers(self.header_map())
	}
}

const LUA_SCRIPT: &str = include_str!("limit.lua");

impl RateLimiter {
	pub async fn new(
		redis: fred::clients::RedisClient,
		limits: HashMap<RateLimitResource, LimitsConfig>,
	) -> anyhow::Result<Self> {
		let lib = fred::types::Library::from_code(&redis, LUA_SCRIPT).await?;

		Ok(Self {
			ratelimit: lib
				.functions()
				.get("api_ratelimit")
				.context("failed to get api_ratelimit function")?
				.clone(),
			redis,
			limits,
		})
	}

	pub async fn acquire(&self, request: RateLimitRequest) -> Result<Option<RateLimitResponse>, ApiError> {
		let Some(limits) = self.limits.get(&request.resource) else {
			return Ok(None);
		};

		let key = format!("ratelimit:{}:{}", request.resource, request.id);

		let limit = request.limit.unwrap_or(limits.requests);
		let ticket_count = request.ticket_count;
		if ticket_count == 0 {
			return Ok(None);
		}

		if limit == 0 {
			return Err(RateLimitResponse {
				resource: request.resource,
				limit,
				remaining: 0,
				reset: 0,
				used: 0,
			}
			.error());
		}

		let ttl = limits.interval_seconds;
		let overuse_threshold = limits.overuse_threshold.unwrap_or(0);
		let overuse_punishment = limits.overuse_punishment.unwrap_or(0);

		let result: Vec<i32> = self
			.ratelimit
			.fcall(
				&self.redis,
				vec![key.as_str()],
				vec![limit, ticket_count, ttl, overuse_threshold, overuse_punishment],
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to call ratelimit function");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let remaining = result[0];
		let reset = result[1].max(0) as u64;

		if remaining < 0 {
			return Err(RateLimitResponse {
				resource: request.resource,
				limit,
				reset,
				remaining: 0,
				used: 0,
			}
			.error());
		}

		Ok(Some(RateLimitResponse {
			limit,
			remaining: remaining as u64,
			reset,
			used: request.ticket_count,
			resource: request.resource,
		}))
	}
}

pub async fn with_ratelimit<'a, R, F>(
	global: &'a Arc<Global>,
	req: RateLimitRequest,
	service: impl FnOnce() -> F,
) -> Response
where
	R: IntoResponse,
	F: std::future::Future<Output = R> + Send + 'a,
{
	match global.rate_limiter.acquire(req).await {
		Ok(Some(response)) => {
			let mut resp = service().await.into_response();

			resp.headers_mut().extend(response.header_map());

			resp
		}
		Ok(None) => service().await.into_response(),
		Err(e) => e.into_response(),
	}
}
