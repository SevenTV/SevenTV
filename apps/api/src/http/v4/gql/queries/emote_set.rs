use std::sync::Arc;

use async_graphql::{Context, Object};
use shared::database::emote_set::EmoteSetId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::EmoteSet;

#[derive(Default)]
pub struct EmoteSetQuery;

#[Object]
impl EmoteSetQuery {
	async fn emote_set(&self, ctx: &Context<'_>, id: EmoteSetId) -> Result<Option<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?;

		Ok(emote_set.map(Into::into))
	}
}
