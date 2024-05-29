// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/emoteset.gql

use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object};
use shared::{
	database::{EmoteSetId, Id, UserId},
	types::old::EmoteSetFlagModel,
};

use crate::{global::Global, http::error::ApiError};

use super::{emotes::Emote, users::UserPartial};

#[derive(Default)]
pub struct EmoteSetsQuery;

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteSet {
	pub id: EmoteSetId,
	pub name: String,
	pub flags: EmoteSetFlagModel,
	pub tags: Vec<String>,
	// emotes
	// emote_count
	// capacity
	pub origins: Vec<EmoteSetOrigin>, // always empty
	pub owner_id: Option<UserId>,
	// owner
}

impl EmoteSet {
	pub fn from_db(value: shared::database::EmoteSet) -> Self {
		Self {
			flags: value.to_old_flags(),
			id: value.id,
			name: value.name,
			tags: value.tags,
			origins: Vec::new(),
			owner_id: value.owner_id,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl EmoteSet {
	async fn emotes(&self) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn emote_count(&self) -> Result<u32, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn capacity<'ctx>(&self, ctx: &Context<'ctx>) -> Result<u16, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let Some(owner_id) = self.owner_id else {
			return Ok(600);
		};

		let user = global
			.user_by_id_loader()
			.load(global, owner_id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = {
			let mut roles = global
				.role_by_id_loader()
				.load_many(user.entitled_cache.role_ids.iter().copied())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

			global_config
				.role_ids
				.iter()
				.filter_map(|id| roles.remove(id))
				.collect::<Vec<_>>()
		};

		Ok(user.compute_permissions(&roles).emote_set_slots_limit.unwrap_or(600))
	}

	async fn owner<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<UserPartial>, ApiError> {
		let Some(id) = self.owner_id else {
			return Ok(None);
		};

		let global = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(Some(UserPartial::load_from_db(global, id).await?))
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSetOrigin {
	pub id: Id<()>,
	pub weight: i32,
	pub slices: Vec<i32>,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetsQuery {
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteSetId) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_set = global
			.emote_set_by_id_loader()
			.load(id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		Ok(EmoteSet::from_db(emote_set))
	}
}