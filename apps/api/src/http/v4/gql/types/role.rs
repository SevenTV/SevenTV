use std::sync::Arc;

use async_graphql::{ComplexObject, Context, SimpleObject};
use shared::database::role::RoleId;
use shared::database::user::UserId;

use super::{Color, User};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

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
	#[tracing::instrument(skip_all, name = "Role::created_by")]
	pub async fn created_by(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
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
