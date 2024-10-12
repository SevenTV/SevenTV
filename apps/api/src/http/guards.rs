use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{Context, Guard};
use shared::database::role::permissions::{Permission, PermissionsExt, RateLimitResource};
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::ratelimit::{RateLimitRequest, RateLimitResponse};

pub struct PermissionGuard {
	pub permissions: Vec<Permission>,
	pub all: bool,
}

impl PermissionGuard {
	pub fn one(permission: impl Into<Permission>) -> Self {
		Self {
			permissions: vec![permission.into()],
			all: true,
		}
	}

	pub fn all(permissions: impl IntoIterator<Item = impl Into<Permission>>) -> Self {
		Self {
			permissions: permissions.into_iter().map(Into::into).collect(),
			all: true,
		}
	}
}

impl Guard for PermissionGuard {
	async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		if self.all {
			if !session.has_all(self.permissions.iter().copied()) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to use this endpoint",
				)
				.into());
			}
		} else if !session.has_any(self.permissions.iter().copied()) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you do not have permission to use this endpoint",
			)
			.into());
		}

		Ok(())
	}
}

pub struct UserGuard(pub UserId);

impl Guard for UserGuard {
	async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;

		if session.user_id() != Some(self.0) {
			return Err(
				ApiError::forbidden(ApiErrorCode::LackingPrivileges, "you are not authorized to use this endpoint").into(),
			);
		}

		Ok(())
	}
}

pub struct RateLimitGuard {
	resource: RateLimitResource,
	ticket_count: i64,
}

impl RateLimitGuard {
	pub fn new(resource: RateLimitResource, ticket_count: i64) -> Self {
		Self { resource, ticket_count }
	}

	pub fn search(ticket_count: i64) -> Self {
		Self::new(RateLimitResource::Search, ticket_count)
	}
}

impl Guard for RateLimitGuard {
	async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let response = ctx.data::<RateLimitResponseStore>().map_err(|_| {
			ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing rate limit response data")
		})?;

		let mut req = RateLimitRequest::new(self.resource, session);

		if req.ticket_count > 0 {
			req.ticket_count = self.ticket_count;
		}

		if let Some(rate_limit) = global.rate_limiter.acquire(req).await? {
			let used = response.incr_used(self.resource, req.ticket_count);

			let headers = RateLimitResponse {
				resource: rate_limit.resource,
				limit: rate_limit.limit,
				remaining: rate_limit.remaining,
				reset: rate_limit.reset,
				used,
			}
			.header_map();

			for (key, value) in headers {
				if let Some(key) = key {
					ctx.insert_http_header(key, value);
				}
			}
		}

		Ok(())
	}
}

#[derive(Clone)]
pub struct RateLimitResponseStore {
	used: Arc<spin::Mutex<HashMap<RateLimitResource, i64>>>,
}

impl RateLimitResponseStore {
	pub fn new() -> Self {
		Self {
			used: Arc::new(spin::Mutex::new(HashMap::new())),
		}
	}

	pub fn incr_used(&self, resource: RateLimitResource, count: i64) -> i64 {
		let mut used = self.used.lock();
		*used.entry(resource).or_insert(0) += count;
		*used.get(&resource).unwrap()
	}
}
