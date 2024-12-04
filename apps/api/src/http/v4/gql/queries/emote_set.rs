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
	#[tracing::instrument(skip_all, name = "EmoteSetQuery::emote_set")]
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

	#[tracing::instrument(skip_all, name = "EmoteSetQuery::emote_sets")]
	async fn emote_sets(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(min_items = 1, max_items = 50))] ids: Vec<EmoteSetId>,
	) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_sets = global
			.emote_set_by_id_loader
			.load_many(ids)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.into_values()
			.map(Into::into)
			.collect();

		Ok(emote_sets)
	}
}
