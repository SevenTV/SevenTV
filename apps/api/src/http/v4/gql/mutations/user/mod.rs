use std::sync::Arc;

use async_graphql::Context;
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod operation;

#[derive(Default)]
pub struct UserMutation;

#[async_graphql::Object]
impl UserMutation {
	#[tracing::instrument(skip_all, name = "UserMutation::user")]
	async fn user(&self, ctx: &Context<'_>, id: UserId) -> Result<operation::UserOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		Ok(operation::UserOperation { user })
	}
}
