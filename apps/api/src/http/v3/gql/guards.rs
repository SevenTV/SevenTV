use std::sync::Arc;

use async_graphql::{Context, Guard};
use shared::database::Permission;

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

		let user = global
			.user_by_id_loader()
			.load(global, auth_session.user_id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::UNAUTHORIZED)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = {
			let mut roles = global
				.role_by_id_loader()
				.load_many(user.entitled_cache.role_ids.iter().copied())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

			global_config
				.role_ids
				.iter()
				.filter_map(|id| roles.remove(id))
				.collect::<Vec<_>>()
		};

		let user_permissions = user.compute_permissions(&roles);

		if self.permissions.iter().any(|p| !user_permissions.has(*p)) {
			return Err(ApiError::FORBIDDEN.into());
		}

		Ok(())
	}
}
