use std::sync::Arc;

use anyhow::Context;
use axum::http::HeaderName;
use axum::response::{IntoResponse, Response};
use hyper::HeaderMap;
use shared::database::role::permissions::{AdminPermission, PermissionsExt, RateLimitResource};
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;

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
	redis: fred::clients::RedisPool,
	ratelimit: fred::types::Function,
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
			RateLimitIdentifier::Ip(ip) => write!(f, "ip:[{}]", ip),
			RateLimitIdentifier::UserId(id) => write!(f, "user:[{}]", id),
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct RateLimitRequest {
	pub resource: RateLimitResource,
	pub id: RateLimitIdentifier,
	pub limit: i64,
	pub interval_seconds: i64,
	pub ticket_count: i64,
	pub punishment_ttl: Option<i64>,
	pub punishment_threshold: Option<i64>,
}

impl RateLimitRequest {
	pub fn new(resource: RateLimitResource, session: &Session) -> Self {
		let limits = session.permissions().ratelimit(resource);

		Self {
			resource,
			id: session
				.user_id()
				.map(RateLimitIdentifier::UserId)
				.unwrap_or(RateLimitIdentifier::Ip(session.ip())),
			limit: limits.map(|l| l.requests).unwrap_or(0),
			ticket_count: if session.has(AdminPermission::BypassRateLimit) { 0 } else { 1 },
			interval_seconds: limits.map(|l| l.interval_seconds).unwrap_or(0),
			punishment_ttl: limits.and_then(|l| l.overuse_punishment),
			punishment_threshold: limits.and_then(|l| l.overuse_threshold),
		}
	}

	pub async fn http<R, F>(self, global: &Arc<Global>, svc: F) -> Result<Response, ApiError>
	where
		R: IntoResponse,
		F: std::future::Future<Output = R>,
	{
		match global.rate_limiter.acquire(self).await {
			Ok(Some(response)) => {
				let mut resp = svc.await.into_response();

				resp.headers_mut().extend(response.header_map());

				Ok(resp)
			}
			Ok(None) => Ok(svc.await.into_response()),
			Err(e) => Err(e),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitResponse {
	pub remaining: i64,
	pub reset: i64,
	pub limit: i64,
	pub used: i64,
	pub resource: RateLimitResource,
}

impl RateLimitResponse {
	pub fn header_map(&self) -> HeaderMap {
		let x_rate_limit_limit =
			HeaderName::try_from(format!("x-ratelimit-{}-limit", self.resource.as_str())).expect("invalid header name");
		let x_rate_limit_remaining =
			HeaderName::try_from(format!("x-ratelimit-{}-remaining", self.resource.as_str())).expect("invalid header name");
		let x_rate_limit_reset =
			HeaderName::try_from(format!("x-ratelimit-{}-reset", self.resource.as_str())).expect("invalid header name");
		let x_rate_limit_used =
			HeaderName::try_from(format!("x-ratelimit-{}-used", self.resource.as_str())).expect("invalid header name");

		HeaderMap::from_iter([
			(x_rate_limit_limit, self.limit.into()),
			(x_rate_limit_remaining, self.remaining.into()),
			(x_rate_limit_reset, self.reset.into()),
			(x_rate_limit_used, self.used.into()),
		])
	}

	pub fn error(&self) -> ApiError {
		ApiError::too_many_requests("rate limit exceeded").with_extra_headers(self.header_map())
	}
}

const LUA_SCRIPT: &str = include_str!("limit.lua");

impl RateLimiter {
	pub async fn new(redis: fred::clients::RedisPool) -> anyhow::Result<Self> {
		let lib = fred::types::Library::from_code(redis.next(), LUA_SCRIPT).await?;

		Ok(Self {
			ratelimit: lib
				.functions()
				.get("api_ratelimit")
				.context("failed to get api_ratelimit function")?
				.clone(),
			redis,
		})
	}

	#[tracing::instrument(skip_all, name = "RateLimiter::acquire", fields(resource = request.resource.as_str()))]
	pub async fn acquire(&self, request: RateLimitRequest) -> Result<Option<RateLimitResponse>, ApiError> {
		if request.ticket_count <= 0 || request.interval_seconds <= 0 {
			return Ok(None);
		}
		if request.limit <= 0 {
			return Err(RateLimitResponse {
				resource: request.resource,
				limit: 0,
				remaining: 0,
				reset: 0,
				used: 0,
			}
			.error());
		}

		let result: Vec<i64> = self
			.ratelimit
			.fcall(
				&self.redis,
				vec![format!("ratelimit:v2:{}:{}", request.resource.as_str(), request.id).as_str()],
				vec![
					request.limit,
					request.ticket_count,
					request.interval_seconds,
					request.punishment_threshold.unwrap_or(0).max(0),
					request.punishment_ttl.unwrap_or(0).max(0),
				],
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to call ratelimit function");
				ApiError::internal_server_error(ApiErrorCode::RateLimitExceeded, "failed to call ratelimit function")
			})?;

		let remaining = result[0];
		let reset = result[1].max(0);

		if remaining < 0 {
			return Err(RateLimitResponse {
				resource: request.resource,
				limit: request.limit,
				reset,
				remaining: -1,
				used: 0,
			}
			.error());
		}

		Ok(Some(RateLimitResponse {
			limit: request.limit,
			remaining,
			reset,
			used: request.ticket_count,
			resource: request.resource,
		}))
	}
}
