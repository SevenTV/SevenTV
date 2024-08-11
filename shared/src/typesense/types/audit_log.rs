use chrono::Utc;

use super::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};
use crate::database::stored_event::{StoredEventData, StoredEventEmoteData, StoredEventEmoteSetData, StoredEventId, StoredEventTicketData, StoredEventUserData, StoredEventUserSessionData};
use crate::database::user::UserId;
use crate::database::{self, Id};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "events")]
#[serde(deny_unknown_fields)]
pub struct Event {
	pub id: StoredEventId,
	pub actor_id: Option<UserId>,
	pub target_id: Id<()>,
	pub kind: TargetKind,
	pub action: ActionKind,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::stored_event::StoredEvent> for Event {
	fn from(value: database::stored_event::StoredEvent) -> Self {
		let (target_id, kind, action) = split_kinds(value.data);

		Self {
			id: value.id,
			actor_id: value.actor_id,
			target_id,
			kind,
			action,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum TargetKind {
	Emote = 0,
	EmoteSet = 1,
	User = 2,
	UserSession = 3,
	Ticket = 4,
}

impl_typesense_type!(TargetKind, Int32);

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum ActionKind {
	Create = 0,
	Modify = 1,
	Delete = 2,
	Merge = 3,

	EmoteProcess = 4,
}

impl_typesense_type!(ActionKind, Int32);

fn split_kinds(data: StoredEventData) -> (Id<()>, TargetKind, ActionKind) {
	match data {
		StoredEventData::Emote { target_id, data } => (
			target_id.cast(),
			TargetKind::Emote,
			match data {
				StoredEventEmoteData::ChangeFlags { .. }
				| StoredEventEmoteData::ChangeName { .. }
				| StoredEventEmoteData::ChangeTags { .. }
				| StoredEventEmoteData::ChangeOwner { .. } => ActionKind::Modify,
				StoredEventEmoteData::Delete { .. } => ActionKind::Delete,
				StoredEventEmoteData::Process { .. } => ActionKind::EmoteProcess,
				StoredEventEmoteData::Upload { .. } => ActionKind::Create,
				StoredEventEmoteData::Merge { .. } => ActionKind::Merge,
			},
		),
		StoredEventData::EmoteSet { target_id, data } => (
			target_id.cast(),
			TargetKind::EmoteSet,
			match data {
				StoredEventEmoteSetData::ChangeCapacity { .. }
				| StoredEventEmoteSetData::ChangeName { .. }
				| StoredEventEmoteSetData::ChangeTags { .. }
				| StoredEventEmoteSetData::AddEmote { .. }
				| StoredEventEmoteSetData::RemoveEmote { .. }
				| StoredEventEmoteSetData::RenameEmote { .. } => ActionKind::Modify,
				StoredEventEmoteSetData::Create { .. } => ActionKind::Create,
				StoredEventEmoteSetData::Delete { .. } => ActionKind::Delete,
			},
		),
		StoredEventData::User { target_id, data } => (
			target_id.cast(),
			TargetKind::User,
			match data {
				StoredEventUserData::Create => ActionKind::Create,
				StoredEventUserData::Merge { .. } => ActionKind::Merge,
				StoredEventUserData::Delete { .. } => ActionKind::Delete,
				_ => ActionKind::Modify,
			},
		),
		StoredEventData::UserSession { target_id, data } => (
			target_id.cast(),
			TargetKind::UserSession,
			match data {
				StoredEventUserSessionData::Create { .. } => ActionKind::Create,
				StoredEventUserSessionData::Delete { .. } => ActionKind::Delete,
			},
		),
		StoredEventData::Ticket { target_id, data } => (
			target_id.cast(),
			TargetKind::Ticket,
			match data {
				StoredEventTicketData::AddMember { .. }
				| StoredEventTicketData::ChangeOpen { .. }
				| StoredEventTicketData::ChangePriority { .. }
				| StoredEventTicketData::RemoveMember { .. } => ActionKind::Modify,
				StoredEventTicketData::Create { .. } => ActionKind::Create,
			},
		),
		_ => todo!(),
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Event>()]
}
