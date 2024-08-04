use scuffle_image_processor_proto::event_callback;

use super::badge::BadgeId;
use super::emote::{EmoteFlags, EmoteId};
use super::emote_moderation_request::EmoteModerationRequestId;
use super::emote_set::EmoteSetId;
use super::entitlement::EntitlementEdgeId;
use super::paint::{PaintData, PaintId};
use super::role::permissions::Permissions;
use super::role::RoleId;
use super::ticket::{TicketId, TicketMessageId, TicketPriority};
use super::user::ban::UserBanId;
use super::user::connection::{Platform, UserConnection};
use super::user::editor::{UserEditorId, UserEditorPermissions};
use super::user::profile_picture::UserProfilePictureId;
use super::user::session::UserSessionId;
use super::user::UserId;
use super::{MongoCollection, MongoGenericCollection};
use crate::database::Id;

pub type EventId = Id<Event>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "events")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct Event {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: EventId,
	pub actor_id: Option<UserId>,
	pub data: EventData,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Event>()]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventData {
	Emote {
		target_id: EmoteId,
		data: EventEmoteData,
	},
	EmoteSet {
		target_id: EmoteSetId,
		data: EventEmoteSetData,
	},
	User {
		target_id: UserId,
		data: EventUserData,
	},
	UserProfilePicture {
		target_id: UserProfilePictureId,
		data: EventUserProfilePictureData,
	},
	UserEditor {
		target_id: UserEditorId,
		data: EventUserEditorData,
	},
	UserBan {
		target_id: UserBanId,
		data: EventUserBanData,
	},
	UserSession {
		target_id: UserSessionId,
		data: EventUserSessionData,
	},
	Ticket {
		target_id: TicketId,
		data: EventTicketData,
	},
	TicketMessage {
		target_id: TicketMessageId,
		data: EventTicketMessageData,
	},
	EmoteModerationRequest {
		target_id: EmoteModerationRequestId,
		data: EventEmoteModerationRequestData,
	},
	Paint {
		target_id: PaintId,
		data: EventPaintData,
	},
	Badge {
		target_id: BadgeId,
		data: EventBadgeData,
	},
	Role {
		target_id: RoleId,
		data: EventRoleData,
	},
	EntitlementEdge {
		target_id: EntitlementEdgeId,
		data: EventEntitlementEdgeData,
	},
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageProcessorEvent {
	Success(event_callback::Success),
	Fail(event_callback::Fail),
	Cancel(event_callback::Cancel),
	Start(event_callback::Start),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventEmoteData {
	Upload,
	Process { event: ImageProcessorEvent },
	ChangeName { old: String, new: String },
	Merge { new_emote_id: EmoteId },
	ChangeOwner { old: UserId, new: UserId },
	ChangeTags { old: Vec<String>, new: Vec<String> },
	ChangeFlags { old: EmoteFlags, new: EmoteFlags },
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventEmoteSetData {
	Create,
	ChangeName {
		old: String,
		new: String,
	},
	ChangeTags {
		added: Vec<String>,
		removed: Vec<String>,
	},
	ChangeCapacity {
		old: Option<i32>,
		new: Option<i32>,
	},
	AddEmote {
		emote_id: EmoteId,
		alias: String,
	},
	RemoveEmote {
		emote_id: EmoteId,
	},
	RenameEmote {
		emote_id: EmoteId,
		old_alias: String,
		new_alias: String,
	},
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventUserData {
	Create,
	ChangeActivePaint { old: Option<PaintId>, new: Option<PaintId> },
	ChangeActiveBadge { old: Option<BadgeId>, new: Option<BadgeId> },
	ChangeActiveEmoteSet { old: Option<EmoteSetId>, new: Option<EmoteSetId> },
	AddConnection { platform: Platform },
	RemoveConnection { connection: UserConnection },
	Merge,
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventUserProfilePictureData {
	Create,
	Process { event: ImageProcessorEvent },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventUserEditorData {
	AddEditor {
		editor_id: UserId,
	},
	RemoveEditor {
		editor_id: UserId,
	},
	EditPermissions {
		old: UserEditorPermissions,
		new: UserEditorPermissions,
	},
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventUserBanData {
	Ban,
	ChangeReason {
		old: String,
		new: String,
	},
	ChangeExpiresAt {
		old: Option<chrono::DateTime<chrono::Utc>>,
		new: Option<chrono::DateTime<chrono::Utc>>,
	},
	ChangeUserBanPermissions {
		old: Permissions,
		new: Permissions,
	},
	Unban,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventUserSessionData {
	Create { platform: Platform },
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventTicketData {
	Create,
	AddMember { member: UserId },
	RemoveMember { member: UserId },
	ChangeOpen { old: bool, new: bool },
	ChangePriority { old: TicketPriority, new: TicketPriority },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventTicketMessageData {
	Create,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventEmoteModerationRequestData {
	Create,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventPaintData {
	Create,
	Process { event: ImageProcessorEvent },
	ChangeName { old: String, new: String },
	ChangeData { old: PaintData, new: PaintData },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventBadgeData {
	Create,
	Process { event: ImageProcessorEvent },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventRoleData {
	Create,
	ChangeName { old: String, new: String },
	ChangeColor { old: i32, new: i32 },
	ChangePermissions { old: Permissions, new: Permissions },
	ChangeRank { old: i32, new: i32 },
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventEntitlementEdgeData {
	Create,
	Delete,
}
