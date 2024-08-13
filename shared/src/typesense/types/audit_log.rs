use chrono::Utc;

use super::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};
use crate::database::stored_event::{
	ImageProcessorEvent, StoredEventBadgeData, StoredEventData, StoredEventEmoteData, StoredEventEmoteModerationRequestData, StoredEventEmoteSetData, StoredEventId, StoredEventPaintData, StoredEventRoleData, StoredEventTicketData, StoredEventUserBanData, StoredEventUserData, StoredEventUserProfilePictureData, StoredEventUserSessionData
};
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

impl TryFrom<database::stored_event::StoredEvent> for Event {
	type Error = ();

	fn try_from(value: database::stored_event::StoredEvent) -> Result<Self, Self::Error> {
		let (target_id, kind, action) = split_kinds(value.data).ok_or(())?;

		Ok(Self {
			id: value.id,
			actor_id: value.actor_id,
			target_id,
			kind,
			action,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		})
	}
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum TargetKind {
	Emote = 0,
	EmoteSet = 1,
	User = 2,
	UserProfilePicture = 3,
	UserBan = 4,
	UserSession = 5,
	Ticket = 6,
	Badge = 7,
	Paint = 8,
	Role = 9,
	EmoteModerationRequest = 10,
}

impl_typesense_type!(TargetKind, Int32);

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum ActionKind {
	Create = 0,
	Modify = 1,
	Delete = 2,
	Merge = 3,
	ImageProcess = 4,
}

impl_typesense_type!(ActionKind, Int32);

fn split_kinds(data: StoredEventData) -> Option<(Id<()>, TargetKind, ActionKind)> {
	let res = match data {
		StoredEventData::Emote { target_id, data } => (
			target_id.cast(),
			TargetKind::Emote,
			match data {
				StoredEventEmoteData::Upload { .. } => ActionKind::Create,
				StoredEventEmoteData::Process {
					event: ImageProcessorEvent::Success(_),
				} => ActionKind::ImageProcess,
				StoredEventEmoteData::Process { .. } => return None,
				StoredEventEmoteData::ChangeName { .. }
				| StoredEventEmoteData::ChangeFlags { .. }
				| StoredEventEmoteData::ChangeTags { .. }
				| StoredEventEmoteData::ChangeOwner { .. } => ActionKind::Modify,
				StoredEventEmoteData::Merge { .. } => ActionKind::Merge,
				StoredEventEmoteData::Delete { .. } => ActionKind::Delete,
			},
		),
		StoredEventData::EmoteSet { target_id, data } => (
			target_id.cast(),
			TargetKind::EmoteSet,
			match data {
				StoredEventEmoteSetData::Create { .. } => ActionKind::Create,
				StoredEventEmoteSetData::ChangeName { .. }
				| StoredEventEmoteSetData::ChangeCapacity { .. }
				| StoredEventEmoteSetData::ChangeTags { .. }
				| StoredEventEmoteSetData::AddEmote { .. }
				| StoredEventEmoteSetData::RemoveEmote { .. }
				| StoredEventEmoteSetData::RenameEmote { .. } => ActionKind::Modify,
				StoredEventEmoteSetData::Delete { .. } => ActionKind::Delete,
			},
		),
		StoredEventData::User { target_id, data } => (
			target_id.cast(),
			TargetKind::User,
			match data {
				StoredEventUserData::Create => ActionKind::Create,
				StoredEventUserData::Merge { .. } => ActionKind::Merge,
				StoredEventUserData::ChangeActivePaint { .. }
				| StoredEventUserData::ChangeActiveBadge { .. }
				| StoredEventUserData::ChangeActiveEmoteSet { .. }
				| StoredEventUserData::AddConnection { .. }
				| StoredEventUserData::RemoveConnection { .. } => ActionKind::Modify,
				StoredEventUserData::Delete { .. } => ActionKind::Delete,
			},
		),
		StoredEventData::UserBan { target_id, data } => (
			target_id.cast(),
			TargetKind::UserBan,
			match data {
				StoredEventUserBanData::Ban => ActionKind::Create,
				StoredEventUserBanData::ChangeReason { .. }
				| StoredEventUserBanData::ChangeExpiresAt { .. }
				| StoredEventUserBanData::ChangeUserBanPermissions { .. } => ActionKind::Modify,
				StoredEventUserBanData::Unban => ActionKind::Delete,
			},
		),
		// TODO: ignored for now
		StoredEventData::UserEditor { .. } => return None,
		StoredEventData::UserProfilePicture { target_id, data } => (
			target_id.cast(),
			TargetKind::UserProfilePicture,
			match data {
				StoredEventUserProfilePictureData::Create => ActionKind::Create,
				StoredEventUserProfilePictureData::Process {
					event: ImageProcessorEvent::Success(_),
				} => ActionKind::ImageProcess,
				StoredEventUserProfilePictureData::Process { .. } => return None,
			},
		),
		// TODO: ignored for now
		StoredEventData::EntitlementEdge { .. } => return None,
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
				StoredEventTicketData::Create { .. } => ActionKind::Create,
				StoredEventTicketData::AddMember { .. }
				| StoredEventTicketData::ChangeOpen { .. }
				| StoredEventTicketData::ChangePriority { .. }
				| StoredEventTicketData::RemoveMember { .. } => ActionKind::Modify,
			},
		),
		// TODO: ignored for now
		StoredEventData::TicketMessage { .. } => return None,
		StoredEventData::Badge { target_id, data } => (
			target_id.cast(),
			TargetKind::Badge,
			match data {
				StoredEventBadgeData::Create { .. } => ActionKind::Create,
				StoredEventBadgeData::Process {
					event: ImageProcessorEvent::Success(_),
				} => ActionKind::ImageProcess,
				StoredEventBadgeData::Process { .. } => return None,
			},
		),
		StoredEventData::Paint { target_id, data } => (
			target_id.cast(),
			TargetKind::Paint,
			match data {
				StoredEventPaintData::Create { .. } => ActionKind::Create,
				StoredEventPaintData::Process {
					event: ImageProcessorEvent::Success(_),
				} => ActionKind::ImageProcess,
				StoredEventPaintData::Process { .. } => return None,
				StoredEventPaintData::ChangeData { .. } | StoredEventPaintData::ChangeName { .. } => ActionKind::Modify,
			},
		),
		StoredEventData::Role { target_id, data } => (
			target_id.cast(),
			TargetKind::Role,
			match data {
				StoredEventRoleData::Create => ActionKind::Create,
				StoredEventRoleData::ChangeName { .. }
				| StoredEventRoleData::ChangeColor { .. }
				| StoredEventRoleData::ChangePermissions { .. }
				| StoredEventRoleData::ChangeRank { .. } => ActionKind::Modify,
				StoredEventRoleData::Delete => ActionKind::Delete,
			},
		),
		StoredEventData::EmoteModerationRequest { target_id, data } => (
			target_id.cast(),
			TargetKind::EmoteModerationRequest,
			match data {
				StoredEventEmoteModerationRequestData::Create { .. } => ActionKind::Create,
			},
		)
	};

	Some(res)
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Event>()]
}
