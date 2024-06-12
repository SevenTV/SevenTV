use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use futures::StreamExt;
use mongodb::bson::doc;
use shared::database::{self, Collection, GlobalConfig};
use shared::old_types::{RoleObjectId, RolePermission};

use super::users::User;
use crate::global::Global;
use crate::http::error::ApiError;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/roles.gql

#[derive(Default)]
pub struct RolesQuery;

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Role {
	id: RoleObjectId,
	name: String,
	color: u32,
	// allowed
	// denied
	position: u32,
	// created_at
	invisible: bool,
	// members
	#[graphql(skip)]
	_allowed: RolePermission,
	#[graphql(skip)]
	_denied: RolePermission,
}

impl Role {
	pub fn from_db(value: database::Role, global_config: &GlobalConfig) -> Self {
		let (allowed, denied) = RolePermission::from_db(value.permissions);

		let position = global_config.role_ids.iter().position(|id| *id == value.id).unwrap_or(0) as u32;

		Self {
			id: value.id.into(),
			name: value.name,
			color: value.color,
			position,
			invisible: false,
			_allowed: allowed,
			_denied: denied,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Role {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.id().timestamp()
	}

	async fn allowed(&self) -> String {
		self._allowed.bits().to_string()
	}

	async fn denied(&self) -> String {
		self._denied.bits().to_string()
	}

	// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/resolvers/role/role.go#L19
	async fn members(&self, _page: Option<u32>, _limit: Option<u32>) -> Vec<User> {
		// not implemented
		vec![]
	}
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesQuery {
	async fn roles<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Role>, ApiError> {
		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = database::Role::collection(global.db())
			.find(doc! {}, None)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.filter_map(|r| async {
				match r {
					Ok(role) => Some(Role::from_db(role, &global_config)),
					Err(e) => {
						tracing::error!(error = %e, "failed to load role");
						None
					}
				}
			})
			.collect()
			.await;

		Ok(roles)
	}

	async fn role<'ctx>(&self, ctx: &Context<'ctx>, id: RoleObjectId) -> Result<Option<Role>, ApiError> {
		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let role = global
			.role_by_id_loader()
			.load(id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(role.map(|r| Role::from_db(r, &global_config)))
	}
}
