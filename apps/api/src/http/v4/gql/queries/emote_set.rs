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

		let mut emote_sets = global
			.emote_set_by_id_loader
			.load_many(ids.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?;

		Ok(ids.iter().filter_map(|id| emote_sets.remove(id)).map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "EmoteSetQuery::global")]
	async fn global(
		&self,
		ctx: &Context<'_>,
	) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let config = global
			.global_config_loader
			.load(())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load global config"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "global config not found"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(config.emote_set_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load global emote set"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "global emote set not found"))?;

		Ok(emote_set.into())
	}
}
