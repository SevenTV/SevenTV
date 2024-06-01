use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use hyper::StatusCode;
use shared::database::{User, UserConnection};
use shared::old_types::{EmoteFlagsModel, EmoteSetFlagModel};

use super::emotes::EmotePartial;
use super::users::UserPartial;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::emote_set_loader::{get_virtual_set_emotes_for_user, virtual_user_set};
use crate::http::v3::gql::object_id::{EmoteObjectId, EmoteSetObjectId, ObjectId, UserObjectId};

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/emoteset.gql

#[derive(Default)]
pub struct EmoteSetsQuery;

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteSet {
	id: EmoteSetObjectId,
	name: String,
	flags: EmoteSetFlagModel,
	tags: Vec<String>,
	// emotes
	// emote_count
	// capacity
	origins: Vec<EmoteSetOrigin>, // always empty
	owner_id: Option<UserObjectId>,
	// owner
	#[graphql(skip)]
	virtual_set_of: Option<(User, u16)>,
}

impl EmoteSet {
	pub fn from_db(value: shared::database::EmoteSet) -> Self {
		Self {
			flags: EmoteSetFlagModel::from_db(&value),
			id: value.id.into(),
			name: value.name,
			tags: value.tags,
			origins: Vec::new(),
			owner_id: value.owner_id.map(Into::into),
			virtual_set_of: None,
		}
	}

	pub async fn virtual_set_for_user(user: User, user_connections: Vec<UserConnection>, slots: u16) -> Self {
		let display_name = user_connections
			.iter()
			.find(|conn| conn.main_connection)
			.map(|c| c.platform_display_name.clone());

		let mut set = Self::from_db(virtual_user_set(user.id, display_name, slots));

		set.virtual_set_of = Some((user, slots));

		set
	}
}

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct ActiveEmote {
	id: EmoteObjectId,
	name: String,
	flags: EmoteFlagsModel,
	timestamp: chrono::DateTime<chrono::Utc>,
	data: EmotePartial,
	actor: Option<UserPartial>,
	origin_id: Option<ObjectId<()>>,
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl EmoteSet {
	async fn emotes<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		limit: Option<u32>,
		_origins: Option<bool>,
	) -> Result<Vec<ActiveEmote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_set_emotes = match &self.virtual_set_of {
			Some((user, slots)) => get_virtual_set_emotes_for_user(global, user, *slots).await?,
			None => global
				.emote_set_emote_by_id_loader()
				.load(*self.id)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.unwrap_or_default(),
		};

		let emote_set_emotes = match limit {
			Some(limit) => emote_set_emotes.into_iter().take(limit as usize).collect(),
			None => emote_set_emotes,
		};

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
			.load(global, *owner_id)
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
		Ok(Some(UserPartial::load_from_db(global, *id).await?))
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSetOrigin {
	id: ObjectId<()>,
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
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteSetObjectId) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_set = global
			.emote_set_by_id_loader()
			.load(*id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if let Some(emote_set) = emote_set {
			// all good
			Ok(EmoteSet::from_db(emote_set))
		} else {
			// this may be a virtual set
			// check if there is a user with the provided id
			let user = global
				.user_by_id_loader()
				.load(global, (*id).cast())
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::NOT_FOUND)?;

			let user_connections = global
				.user_connection_by_user_id_loader()
				.load(user.id)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.unwrap_or_default();

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

			let slots = user.compute_permissions(&roles).emote_set_slots_limit.unwrap_or(600);

			Ok(EmoteSet::virtual_set_for_user(user, user_connections, slots).await)
		}
	}

	#[graphql(name = "emoteSetsByID")]
	async fn emote_sets_by_id<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		list: Vec<EmoteSetObjectId>,
	) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if list.len() > 1000 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "list too large"));
		}

		let mut emote_sets: Vec<_> = global
			.emote_set_by_id_loader()
			.load_many(list.iter().map(|id| **id))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.map(EmoteSet::from_db)
			.collect();

		// load users with ids for virtual sets
		let users = global
			.user_by_id_loader()
			.load_many(global, list.iter().map(|id| (**id).cast()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let user_connections = global
			.user_connection_by_user_id_loader()
			.load_many(users.keys().copied())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = {
			let mut roles = global
				.role_by_id_loader()
				.load_many(users.values().flat_map(|user| user.entitled_cache.role_ids.iter().copied()))
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

			global_config
				.role_ids
				.iter()
				.filter_map(|id| roles.remove(id))
				.collect::<Vec<_>>()
		};

		for (id, user) in users {
			let slots = user.compute_permissions(&roles).emote_set_slots_limit.unwrap_or(600);
			let set =
				EmoteSet::virtual_set_for_user(user, user_connections.get(&id).cloned().unwrap_or_default(), slots).await;
			emote_sets.push(set);
		}

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

				let global_emote_set = global_config.emote_set_ids.first().ok_or(ApiError::NOT_FOUND)?;

				let global_set = global
					.emote_set_by_id_loader()
					.load(*global_emote_set)
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::NOT_FOUND)?;

				Ok(EmoteSet::from_db(global_set))
			}
		}
	}
}
