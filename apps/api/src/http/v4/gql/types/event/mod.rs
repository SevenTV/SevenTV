use async_graphql::OutputType;
use shared::database::user::UserId;
use shared::database::{stored_event, Id};

mod emote;

pub type EmoteEvent = Event<emote::EventEmoteData>;

#[derive(async_graphql::SimpleObject)]
#[graphql(concrete(name = "EmoteEvent", params(emote::EventEmoteData)))]
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
