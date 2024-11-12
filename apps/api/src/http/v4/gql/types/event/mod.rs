use std::sync::Arc;

use async_graphql::{Context, OutputType};
use shared::database::user::UserId;
use shared::database::{stored_event, Id};

use super::{Emote, User};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod emote;
mod user;

pub type EmoteEvent = Event<emote::EventEmoteData>;
pub type UserEvent = Event<user::EventUserData>;

#[derive(async_graphql::SimpleObject)]
#[graphql(complex, concrete(name = "EmoteEvent", params(emote::EventEmoteData)))]
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

#[async_graphql::ComplexObject]
impl EmoteEvent {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}

	async fn target(&self, ctx: &Context<'_>) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote = global
			.emote_by_id_loader
			.load(self.target_id.cast())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))?;

		Ok(Emote::from_db(emote, &global.config.api.cdn_origin))
	}

	async fn actor(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let Some(user_id) = self.actor_id else {
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
}
