use std::sync::Arc;

use async_graphql::{Context, Object};
use shared::database::emote::EmoteId;

use crate::{global::Global, http::{error::ApiError, v4::gql::types::Emote}};

#[derive(Default)]
pub struct EmoteQuery;

#[Object]
impl EmoteQuery {
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteId) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

        let emote = global
            .emote_by_id_loader
            .load(id)
            .await
            .map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
            .ok_or(ApiError::NOT_FOUND)?;

		Ok(emote.into())
	}

	async fn search<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		query: String,
		page: Option<u32>,
		limit: Option<u32>,
	) -> Result<Vec<Emote>, ApiError> {
		todo!()
	}
}
