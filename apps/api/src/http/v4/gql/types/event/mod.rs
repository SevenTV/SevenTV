use std::sync::Arc;

use async_graphql::{Context, OutputType};
use shared::database::user::UserId;
use shared::database::{stored_event, Id};

use super::{Emote, EmoteSet, User};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod emote;
mod emote_set;
mod user;

pub type EmoteEvent = Event<emote::EventEmoteData>;
pub type EmoteSetEvent = Event<emote_set::EventEmoteSetData>;
pub type UserEvent = Event<user::EventUserData>;

#[derive(async_graphql::Union)]
pub enum AnyEvent {
	Emote(EmoteEvent),
	EmoteSet(EmoteSetEvent),
	User(UserEvent),
}

impl TryFrom<stored_event::StoredEvent> for AnyEvent {
	type Error = ();

	fn try_from(value: stored_event::StoredEvent) -> Result<Self, Self::Error> {
		match value.data {
			stored_event::StoredEventData::Emote { .. } => EmoteEvent::try_from(value).map(Self::Emote),
			stored_event::StoredEventData::EmoteSet { .. } => EmoteSetEvent::try_from(value).map(Self::EmoteSet),
			stored_event::StoredEventData::User { .. } => UserEvent::try_from(value).map(Self::User),
			_ => Err(()),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
#[graphql(
	complex,
	concrete(name = "EmoteEvent", params(emote::EventEmoteData)),
	concrete(name = "EmoteSetEvent", params(emote_set::EventEmoteSetData)),
	concrete(name = "UserEvent", params(user::EventUserData))
)]
pub struct Event<T: OutputType> {
	pub id: stored_event::StoredEventId,
	pub actor_id: Option<UserId>,
	pub target_id: Id,
	pub data: T,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<stored_event::StoredEvent> for EmoteEvent {
	type Error = ();

	fn try_from(value: stored_event::StoredEvent) -> Result<Self, Self::Error> {
		let stored_event::StoredEventData::Emote { target_id, data } = value.data else {
			return Err(());
		};

		if let stored_event::StoredEventEmoteData::ChangeFlags { old, new } = data {
			if old == new {
				// Skip events where flags didn't change
				return Err(());
			}
		}

		Ok(Self {
			id: value.id,
			actor_id: value.actor_id,
			target_id: target_id.cast(),
			data: data.into(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		})
	}
}

impl TryFrom<stored_event::StoredEvent> for EmoteSetEvent {
	type Error = ();

	fn try_from(value: stored_event::StoredEvent) -> Result<Self, Self::Error> {
		let stored_event::StoredEventData::EmoteSet { target_id, data } = value.data else {
			return Err(());
		};

		Ok(Self {
			id: value.id,
			actor_id: value.actor_id,
			target_id: target_id.cast(),
			data: data.into(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		})
	}
}

impl TryFrom<stored_event::StoredEvent> for UserEvent {
	type Error = ();

	fn try_from(value: stored_event::StoredEvent) -> Result<Self, Self::Error> {
		let stored_event::StoredEventData::User { target_id, data } = value.data else {
			return Err(());
		};

		Ok(Self {
			id: value.id,
			actor_id: value.actor_id,
			target_id: target_id.cast(),
			data: data.try_into()?,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		})
	}
}

async fn actor<T: OutputType>(event: &Event<T>, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
	let Some(user_id) = event.actor_id else {
		return Ok(None);
	};

	let global: &Arc<Global> = ctx
		.data()
		.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

	let user = global
		.user_loader
		.load(global, user_id)
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

	Ok(user.map(Into::into))
}

#[async_graphql::ComplexObject]
impl EmoteEvent {
	#[tracing::instrument(skip_all, name = "EmoteEvent::created_at")]
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	#[tracing::instrument(skip_all, name = "EmoteEvent::target")]
	async fn target(&self, ctx: &Context<'_>) -> Result<Option<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load(self.target_id.cast())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?;

		Ok(emote.map(|e| Emote::from_db(e, &global.config.api.cdn_origin)))
	}

	#[tracing::instrument(skip_all, name = "EmoteEvent::actor")]
	async fn actor(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		actor(self, ctx).await
	}
}

#[async_graphql::ComplexObject]
impl EmoteSetEvent {
	#[tracing::instrument(skip_all, name = "EmoteSetEvent::created_at")]
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	#[tracing::instrument(skip_all, name = "EmoteSetEvent::target")]
	async fn target(&self, ctx: &Context<'_>) -> Result<Option<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(self.target_id.cast())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?;

		Ok(emote_set.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "EmoteSetEvent::actor")]
	async fn actor(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		actor(self, ctx).await
	}
}

#[async_graphql::ComplexObject]
impl UserEvent {
	#[tracing::instrument(skip_all, name = "UserEvent::created_at")]
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	#[tracing::instrument(skip_all, name = "UserEvent::target")]
	async fn target(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.target_id.cast())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "UserEvent::actor")]
	async fn actor(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		actor(self, ctx).await
	}
}
