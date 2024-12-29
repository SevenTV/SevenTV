use std::sync::Arc;

use async_graphql::Context;
use shared::database::product::codes::RedeemCodeId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod operation;

#[derive(Default)]
pub struct RedeemCodeMutation;

#[async_graphql::Object]
impl RedeemCodeMutation {
	#[tracing::instrument(skip_all, name = "RedeemCodeMutation::redeem_code")]
	async fn redeem_code(&self, ctx: &Context<'_>, id: RedeemCodeId) -> Result<operation::RedeemCodeOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let code = global
			.redeem_code_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		Ok(operation::RedeemCodeOperation { code })
	}
}
