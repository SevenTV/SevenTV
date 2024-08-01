use super::badge::BadgeId;
use super::emote::{EmoteFlags, EmoteId};
use super::emote_moderation_request::EmoteModerationRequestId;
use super::emote_set::EmoteSetId;
use super::entitlement::EntitlementEdgeId;
use super::paint::{PaintData, PaintId};
use super::role::permissions::Permissions;
use super::role::RoleId;
use super::ticket::{TicketId, TicketMessageId};
use super::user::ban::UserBanId;
use super::user::connection::Platform;
use super::user::editor::{UserEditorId, UserEditorPermissions};
use super::user::session::UserSessionId;
use super::user::UserId;
use super::{MongoCollection, MongoGenericCollection};
use crate::database::Id;

pub type AuditLogId = Id<AuditLog>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "audit_logs")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct AuditLog {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: AuditLogId,
	pub actor_id: Option<UserId>,
	pub data: AuditLogData,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<AuditLog>()]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogData {
	Emote {
		target_id: EmoteId,
		data: AuditLogEmoteData,
	},
	EmoteSet {
		target_id: EmoteSetId,
		data: AuditLogEmoteSetData,
	},
	User {
		target_id: UserId,
		data: AuditLogUserData,
	},
	UserEditor {
		target_id: UserEditorId,
		data: AuditLogUserEditorData,
	},
	UserBan {
		target_id: UserBanId,
		data: AuditLogUserBanData,
	},
	UserSession {
		target_id: UserSessionId,
		data: AuditLogUserSessionData,
	},
	Ticket {
		target_id: TicketId,
		data: AuditLogTicketData,
	},
	TicketMessage {
		target_id: TicketMessageId,
		data: AuditLogTicketMessageData,
	},
	EmoteModerationRequest {
		target_id: EmoteModerationRequestId,
		data: AuditLogEmoteModerationRequestData,
	},
	Paint {
		target_id: PaintId,
		data: AuditLogPaintData,
	},
	Badge {
		target_id: BadgeId,
		data: AuditLogBadgeData,
	},
	Role {
		target_id: RoleId,
		data: AuditLogRoleData,
	},
	EntitlementEdge {
		target_id: EntitlementEdgeId,
		data: AuditLogEntitlementEdgeData,
	},
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogEmoteData {
	Upload,
	Process,
	ChangeName { old: String, new: String },
	Merge { new_emote_id: EmoteId },
	ChangeOwner { old: UserId, new: UserId },
	ChangeTags { old: Vec<String>, new: Vec<String> },
	ChangeFlags { old: EmoteFlags, new: EmoteFlags },
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogEmoteSetData {
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
		old_name: String,
		new_name: String,
	},
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogUserData {
	Create,
	ChangeActivePaint { paint_id: PaintId },
	ChangeActiveBadge { badge_id: BadgeId },
	ChangeActiveEmoteSet { emote_set_id: EmoteSetId },
	UploadProfilePicture,
	ProcessProfilePicture,
	AddConnection { platform: Platform },
	RemoveConnection { platform: Platform },
	Merge,
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogUserEditorData {
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
pub enum AuditLogUserBanData {
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
pub enum AuditLogUserSessionData {
	Login { platform: Platform },
	Logout,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogTicketData {
	Create,
	AddMember { member: UserId },
	RemoveMember { member: UserId },
	ChangeOpen { old: bool, new: bool },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogTicketMessageData {
	Create,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogEmoteModerationRequestData {
	Create,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogPaintData {
	Create,
	Process,
	ChangeName { old: String, new: String },
	ChangeData { old: PaintData, new: PaintData },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogBadgeData {
	Create,
	Process,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogRoleData {
	Create,
	ChangeName {
		old: String,
		new: String,
	},
	ChangeColor {
		old: i32,
		new: i32,
	},
	ChangePermissions {
		old: Permissions,
		new: Permissions,
	},
	ChangeRank {
		old: i32,
		new: i32,
	},
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogEntitlementEdgeData {
	Create,
}
