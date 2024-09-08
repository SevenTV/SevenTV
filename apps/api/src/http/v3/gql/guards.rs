use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::{Context, Guard};
use shared::database::role::permissions::{Permission, PermissionsExt};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::ratelimit::{RateLimitRequest, RateLimitResource, RateLimitResponse};

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
		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await?;

		if self.all {
			if !user.has_all(self.permissions.iter().copied()) {
				return Err(ApiError::FORBIDDEN.into());
			}
		} else if !user.has_any(self.permissions.iter().copied()) {
			return Err(ApiError::FORBIDDEN.into());
		}

		Ok(())
	}
}

pub struct RateLimitGuard {
	resource: RateLimitResource,
	ticket_count: u64,
}

impl RateLimitGuard {
	pub fn new(resource: RateLimitResource, ticket_count: u64) -> Self {
		Self { resource, ticket_count }
	}

	pub fn search(ticket_count: u64) -> Self {
		Self::new(RateLimitResource::Search, ticket_count)
	}
}

impl Guard for RateLimitGuard {
	async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let ip = ctx.data::<std::net::IpAddr>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let response = ctx
			.data::<Arc<RateLimitResponseStore>>()
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let authed_user = if let Some(auth_session) = ctx.data_opt::<AuthSession>() {
			Some(auth_session.user(global).await.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?)
		} else {
			None
		};

		let mut req = RateLimitRequest::new(self.resource, authed_user, *ip).with_ticket_count(self.ticket_count);

		if let Some(gql_limit) = authed_user.and_then(|s| s.computed.permissions.graphql_rate_limit) {
			req = req.with_limit(gql_limit.max(0) as u64);
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

pub struct RateLimitResponseStore {
	used: spin::Mutex<HashMap<RateLimitResource, u64>>,
}

impl RateLimitResponseStore {
	pub fn new() -> Self {
		Self {
			used: spin::Mutex::new(HashMap::new()),
		}
	}

	pub fn incr_used(&self, resource: RateLimitResource, count: u64) -> u64 {
		let mut used = self.used.lock();
		*used.entry(resource).or_insert(0) += count;
		*used.get(&resource).unwrap()
	}
}
