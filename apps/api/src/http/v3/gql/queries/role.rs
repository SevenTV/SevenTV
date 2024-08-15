use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use futures::{TryFutureExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use shared::database::queries::filter;
use shared::database::role::Role as DbRole;
use shared::database::MongoCollection;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::role_permission::RolePermission;

use super::user::User;
use crate::global::Global;
use crate::http::error::ApiError;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/roles.gql

#[derive(Default)]
pub struct RolesQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Role {
	id: GqlObjectId,
	name: String,
	color: i32,
	// allowed
	// denied
	position: i32,
	// created_at
	invisible: bool,
	// members
	#[graphql(skip)]
	_allowed: RolePermission,
	#[graphql(skip)]
	_denied: RolePermission,
}

impl Role {
	pub fn from_db(value: shared::database::role::Role) -> Self {
		let (allowed, denied) = RolePermission::from_db(value.permissions);

		Self {
			id: value.id.into(),
			name: value.name,
			color: value.color.unwrap_or_default(),
			position: value.rank,
			invisible: false,
			_allowed: allowed,
			_denied: denied,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Role {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
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

		let roles: Vec<DbRole> = DbRole::collection(&global.db)
			.find(filter::filter!(DbRole {}))
			.with_options(FindOptions::builder().sort(doc! { "rank": -1 }).build())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(roles.into_iter().map(Role::from_db).collect())
	}

	async fn role<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<Role>, ApiError> {
		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let role = global
			.role_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(role.map(|r| Role::from_db(r)))
	}
}
