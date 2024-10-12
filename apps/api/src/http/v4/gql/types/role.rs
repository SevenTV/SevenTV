use std::sync::Arc;

use async_graphql::{ComplexObject, Context, SimpleObject};
use shared::database::{role::RoleId, user::UserId};

use crate::{
	global::Global,
	http::error::{ApiError, ApiErrorCode},
};

use super::{Color, User};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Role {
	pub id: RoleId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by_id: UserId,
	pub color: Option<Color>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<shared::database::role::Role> for Role {
	fn from(value: shared::database::role::Role) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			created_by_id: value.created_by,
			color: value.color.map(Color),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[ComplexObject]
impl Role {
	pub async fn created_by<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.created_by_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}
}
