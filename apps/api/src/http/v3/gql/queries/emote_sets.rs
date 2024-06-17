use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use hyper::StatusCode;
use mongodb::bson::doc;
use shared::database::emote_set::EmoteSetEmote;
use shared::database::user::UserId;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{ActiveEmoteFlagModel, EmoteSetFlagModel};

use super::emotes::{Emote, EmotePartial};
use super::users::UserPartial;
use crate::dataloader::user_loader::{load_user_and_permissions, load_users_and_permissions};
use crate::global::Global;
use crate::http::error::ApiError;

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
			capacity: value.capacity,
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
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote = global
			.emote_by_id_loader()
			.load(self.id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote
			.map(|e| Emote::from_db(global, e))
			.unwrap_or_else(Emote::deleted_emote)
			.into())
	}

	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if let Some(actor_id) = self.actor_id {
			Ok(UserPartial::load_from_db(global, actor_id).await?)
		} else {
			Ok(None)
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl EmoteSet {
	async fn emotes(&self, limit: Option<u32>, origins: Option<bool>) -> Result<Vec<ActiveEmote>, ApiError> {
		Ok(self
			.emotes
			.iter()
			.filter(|emote| match origins {
				Some(true) => emote.origin_set_id.is_some(),
				Some(false) => emote.origin_set_id.is_none(),
				None => true,
			})
			.take(limit.unwrap_or(100000) as usize)
			.cloned()
			.map(|emote| ActiveEmote::from_db(emote))
			.collect())
	}

	async fn emote_count(&self) -> Result<u32, ApiError> {
		Ok(self.emotes.len() as u32)
	}

	async fn owner<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<UserPartial>, ApiError> {
		let Some(id) = self.owner_id else {
			return Ok(None);
		};

		let global = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(Some(UserPartial::load_from_db(global, id.0.cast()).await?))
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
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_set = global
			.emote_set_by_id_loader()
			.load(id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		Ok(EmoteSet::from_db(emote_set))
	}

	#[graphql(name = "emoteSetsByID")]
	async fn emote_sets_by_id<'ctx>(&self, ctx: &Context<'ctx>, list: Vec<GqlObjectId>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		// TODO: 1000 is very large
		if list.len() > 1000 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "list too large"));
		}

		let emote_sets: Vec<_> = global
			.emote_set_by_id_loader()
			.load_many(list.into_iter().map(|id| id.0.cast()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.map(EmoteSet::from_db)
			.collect();

		Ok(emote_sets)
	}

	async fn named_emote_set<'ctx>(&self, ctx: &Context<'ctx>, name: EmoteSetName) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		match name {
			EmoteSetName::Global => {
				let global_config = global
					.global_config_loader()
					.load(())
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

				let global_set = global
					.emote_set_by_id_loader()
					.load(global_config.emote_set_id)
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::NOT_FOUND)?;

				Ok(EmoteSet::from_db(global_set))
			}
		}
	}
}
