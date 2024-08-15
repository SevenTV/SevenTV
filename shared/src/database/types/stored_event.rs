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
use super::user::connection::Platform;
use super::user::editor::{UserEditorId, UserEditorPermissions};
use super::user::profile_picture::UserProfilePictureId;
use super::user::session::UserSessionId;
use super::user::UserId;
use super::{MongoCollection, MongoGenericCollection};
use crate::database::Id;

pub type StoredEventId = Id<StoredEvent>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "events")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct StoredEvent {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: StoredEventId,
	pub actor_id: Option<UserId>,
	pub data: StoredEventData,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<StoredEvent>()]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventData {
	Emote {
		target_id: EmoteId,
		data: StoredEventEmoteData,
	},
	EmoteSet {
		target_id: EmoteSetId,
		data: StoredEventEmoteSetData,
	},
	User {
		target_id: UserId,
		data: StoredEventUserData,
	},
	UserProfilePicture {
		target_id: UserProfilePictureId,
		data: StoredEventUserProfilePictureData,
	},
	UserEditor {
		target_id: UserEditorId,
		data: StoredEventUserEditorData,
	},
	UserBan {
		target_id: UserBanId,
		data: StoredEventUserBanData,
	},
	UserSession {
		target_id: UserSessionId,
		data: StoredEventUserSessionData,
	},
	Ticket {
		target_id: TicketId,
		data: StoredEventTicketData,
	},
	TicketMessage {
		target_id: TicketMessageId,
		data: StoredEventTicketMessageData,
	},
	EmoteModerationRequest {
		target_id: EmoteModerationRequestId,
		data: StoredEventEmoteModerationRequestData,
	},
	Paint {
		target_id: PaintId,
		data: StoredEventPaintData,
	},
	Badge {
		target_id: BadgeId,
		data: StoredEventBadgeData,
	},
	Role {
		target_id: RoleId,
		data: StoredEventRoleData,
	},
	EntitlementEdge {
		target_id: EntitlementEdgeId,
		data: StoredEventEntitlementEdgeData,
	},
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageProcessorEvent {
	/// The event callback is only None for old events
	Success(Option<event_callback::Success>),
	Fail(event_callback::Fail),
	Cancel(event_callback::Cancel),
	Start(event_callback::Start),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventEmoteData {
	Upload,
	Process { event: ImageProcessorEvent },
	ChangeName { old: String, new: String },
	Merge { new_emote_id: EmoteId },
	ChangeOwner { old: UserId, new: UserId },
	ChangeTags { old: Vec<String>, new: Vec<String> },
	ChangeFlags { old: EmoteFlags, new: EmoteFlags },
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventEmoteSetData {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventUserData {
	Create,
	ChangeActivePaint {
		old: Option<PaintId>,
		new: Option<PaintId>,
	},
	ChangeActiveBadge {
		old: Option<BadgeId>,
		new: Option<BadgeId>,
	},
	ChangeActiveEmoteSet {
		old: Option<EmoteSetId>,
		new: Option<EmoteSetId>,
	},
	AddConnection {
		platform: Platform,
	},
	RemoveConnection {
		platform: Platform,
	},
	Merge,
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventUserProfilePictureData {
	Create,
	Process { event: ImageProcessorEvent },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventUserEditorData {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventUserBanData {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventUserSessionData {
	Create { platform: Platform },
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventTicketData {
	Create,
	AddMember { member: UserId },
	RemoveMember { member: UserId },
	ChangeOpen { old: bool, new: bool },
	ChangePriority { old: TicketPriority, new: TicketPriority },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventTicketMessageData {
	Create,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventEmoteModerationRequestData {
	Create,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventPaintData {
	Create,
	Process { event: ImageProcessorEvent },
	ChangeName { old: String, new: String },
	ChangeData { old: PaintData, new: PaintData },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventBadgeData {
	Create,
	Process { event: ImageProcessorEvent },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventRoleData {
	Create,
	ChangeName { old: String, new: String },
	ChangeColor { old: i32, new: i32 },
	ChangePermissions { old: Permissions, new: Permissions },
	ChangeRank { old: i32, new: i32 },
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEventEntitlementEdgeData {
	Create,
	Delete,
}
