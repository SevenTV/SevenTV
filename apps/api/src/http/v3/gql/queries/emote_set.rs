use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use mongodb::bson::doc;
use shared::database::emote_set::EmoteSetEmote;
use shared::database::user::UserId;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{ActiveEmoteFlagModel, EmoteSetFlagModel};

use super::emote::{Emote, EmotePartial};
use super::user::UserPartial;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/emoteset.gql

#[derive(Default)]
pub struct EmoteSetsQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteSet {
	id: GqlObjectId,
	name: String,
	flags: EmoteSetFlagModel,
	tags: Vec<String>,
	#[graphql(skip)]
	emotes: Vec<EmoteSetEmote>,
	// emote_count
	// capacity
	origins: Vec<EmoteSetOrigin>,
	owner_id: Option<GqlObjectId>,
	capacity: i32,
}

impl EmoteSet {
	pub fn from_db(value: shared::database::emote_set::EmoteSet) -> Self {
		Self {
			flags: EmoteSetFlagModel::from_db(&value),
			id: value.id.into(),
			name: value.name,
			tags: value.tags,
			origins: Vec::new(),
			emotes: value.emotes,
			owner_id: value.owner_id.map(Into::into),
			capacity: value.capacity.unwrap_or_default(),
		}
	}
}

#[derive(Debug, Clone, SimpleObject, serde::Deserialize, serde::Serialize)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct ActiveEmote {
	pub id: GqlObjectId,
	pub name: String,
	pub flags: ActiveEmoteFlagModel,
	// timestamp
	// data
	// actor
	pub origin_id: Option<GqlObjectId>,

	#[graphql(skip)]
	pub actor_id: Option<UserId>,
}

impl ActiveEmote {
	pub fn from_db(value: EmoteSetEmote) -> Self {
		Self {
			id: value.id.into(),
			name: value.alias,
			flags: value.flags.into(),
			actor_id: value.added_by_id,
			origin_id: value.origin_set_id.map(Into::into),
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl ActiveEmote {
	async fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	async fn data<'ctx>(&self, ctx: &Context<'ctx>) -> Result<EmotePartial, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?;

		Ok(emote
			.map(|e| Emote::from_db(global, e))
			.unwrap_or_else(Emote::deleted_emote)
			.into())
	}

	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let Some(actor_id) = self.actor_id else {
			return Ok(None);
		};

		Ok(global
			.user_loader
			.load_fast(global, actor_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.map(|u| UserPartial::from_db(global, u)))
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl EmoteSet {
	async fn emotes(&self, limit: Option<u32>, _origins: Option<bool>) -> Result<Vec<ActiveEmote>, ApiError> {
		Ok(self
			.emotes
			.iter()
			.take(limit.unwrap_or(100000) as usize)
			.cloned()
			.map(ActiveEmote::from_db)
			.collect())
	}

	async fn emote_count(&self) -> Result<u32, ApiError> {
		Ok(self.emotes.len() as u32)
	}

	async fn owner<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<UserPartial>, ApiError> {
		let Some(id) = self.owner_id else {
			return Ok(None);
		};

		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load_fast(global, id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.map(|u| UserPartial::from_db(global, u)))
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSetOrigin {
	id: GqlObjectId,
	weight: i32,
	slices: Vec<i32>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum EmoteSetName {
	Global,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetsQuery {
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))?;

		Ok(EmoteSet::from_db(emote_set))
	}

	#[graphql(name = "emoteSetsByID")]
	async fn emote_sets_by_id<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(max_items = 300))] list: Vec<GqlObjectId>,
	) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_sets: Vec<_> = global
			.emote_set_by_id_loader
			.load_many(list.into_iter().map(|id| id.id()))
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.into_values()
			.map(EmoteSet::from_db)
			.collect();

		Ok(emote_sets)
	}

	async fn named_emote_set<'ctx>(&self, ctx: &Context<'ctx>, name: EmoteSetName) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		match name {
			EmoteSetName::Global => {
				let global_config = global
					.global_config_loader
					.load(())
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load global config"))?
					.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "global config not found"))?;

				let global_set = global
					.emote_set_by_id_loader
					.load(global_config.emote_set_id)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))?;

				Ok(EmoteSet::from_db(global_set))
			}
		}
	}
}
