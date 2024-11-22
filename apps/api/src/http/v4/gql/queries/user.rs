use std::sync::Arc;

use async_graphql::{Context, Object};
use shared::database::role::permissions::{FlagPermission, PermissionsExt, UserPermission};
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::User;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
	async fn me(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let Some(user_id) = session.user_id() else {
			return Ok(None);
		};

		let user = global
			.user_loader
			.load(global, user_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}

	async fn user(&self, ctx: &Context<'_>, id: UserId) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let Some(user) = global
			.user_loader
			.load(global, id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		else {
			return Ok(None);
		};

		if user.has(FlagPermission::Hidden) && Some(user.id) != session.user_id() && !session.has(UserPermission::ViewHidden)
		{
			return Ok(None);
		}

		Ok(Some(user.into()))
	}
}
