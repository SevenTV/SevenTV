use std::sync::Arc;

use async_graphql::{Context, Guard};
use shared::database::role::permissions::Permission;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;

pub struct PermissionGuard {
	pub permissions: Vec<Permission>,
}

impl PermissionGuard {
	pub fn one(permission: impl Into<Permission>) -> Self {
		Self {
			permissions: vec![permission.into()],
		}
	}

	pub fn all(permissions: impl IntoIterator<Item = impl Into<Permission>>) -> Self {
		Self {
			permissions: permissions.into_iter().map(Into::into).collect(),
		}
	}
}

impl Guard for PermissionGuard {
	async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await?;

		if self.permissions.iter().any(|p| !user.computed.permissions.has(*p)) {
			return Err(ApiError::FORBIDDEN.into());
		}

		Ok(())
	}
}
