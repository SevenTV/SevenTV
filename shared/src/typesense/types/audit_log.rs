use chrono::Utc;

use super::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};
use crate::database::event::{EventData, EventEmoteData, EventEmoteSetData, EventId, EventTicketData, EventUserData, EventUserSessionData};
use crate::database::user::UserId;
use crate::database::{self, Id};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "events")]
#[serde(deny_unknown_fields)]
pub struct Event {
	pub id: EventId,
	pub actor_id: Option<UserId>,
	pub target_id: Id<()>,
	pub kind: TargetKind,
	pub action: ActionKind,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::event::Event> for Event {
	fn from(value: database::event::Event) -> Self {
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

fn split_kinds(data: EventData) -> (Id<()>, TargetKind, ActionKind) {
	match data {
		EventData::Emote { target_id, data } => (
			target_id.cast(),
			TargetKind::Emote,
			match data {
				EventEmoteData::ChangeFlags { .. }
				| EventEmoteData::ChangeName { .. }
				| EventEmoteData::ChangeTags { .. }
				| EventEmoteData::ChangeOwner { .. } => ActionKind::Modify,
				EventEmoteData::Delete { .. } => ActionKind::Delete,
				EventEmoteData::Process { .. } => ActionKind::EmoteProcess,
				EventEmoteData::Upload { .. } => ActionKind::Create,
				EventEmoteData::Merge { .. } => ActionKind::Merge,
			},
		),
		EventData::EmoteSet { target_id, data } => (
			target_id.cast(),
			TargetKind::EmoteSet,
			match data {
				EventEmoteSetData::ChangeCapacity { .. }
				| EventEmoteSetData::ChangeName { .. }
				| EventEmoteSetData::ChangeTags { .. }
				| EventEmoteSetData::AddEmote { .. }
				| EventEmoteSetData::RemoveEmote { .. }
				| EventEmoteSetData::RenameEmote { .. } => ActionKind::Modify,
				EventEmoteSetData::Create { .. } => ActionKind::Create,
				EventEmoteSetData::Delete { .. } => ActionKind::Delete,
			},
		),
		EventData::User { target_id, data } => (
			target_id.cast(),
			TargetKind::User,
			match data {
				EventUserData::Create => ActionKind::Create,
				EventUserData::Merge { .. } => ActionKind::Merge,
				EventUserData::Delete { .. } => ActionKind::Delete,
				_ => ActionKind::Modify,
			},
		),
		EventData::UserSession { target_id, data } => (
			target_id.cast(),
			TargetKind::UserSession,
			match data {
				EventUserSessionData::Create { .. } => ActionKind::Create,
				EventUserSessionData::Delete { .. } => ActionKind::Delete,
			},
		),
		EventData::Ticket { target_id, data } => (
			target_id.cast(),
			TargetKind::Ticket,
			match data {
				EventTicketData::AddMember { .. }
				| EventTicketData::ChangeOpen { .. }
				| EventTicketData::RemoveMember { .. } => ActionKind::Modify,
				EventTicketData::Create { .. } => ActionKind::Create,
			},
		),
		_ => todo!(),
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Event>()]
}
