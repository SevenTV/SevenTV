use std::sync::Arc;

use async_graphql::Context;
use shared::database::emote_set::EmoteSetId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod operation;

#[derive(Default)]
pub struct EmoteSetMutation;

#[async_graphql::Object]
impl EmoteSetMutation {
	async fn emote_set(&self, ctx: &Context<'_>, id: EmoteSetId) -> Result<operation::EmoteSetOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))?;

		Ok(operation::EmoteSetOperation { emote_set })
	}
}
