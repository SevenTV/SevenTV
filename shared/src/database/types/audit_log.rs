use super::emote::{EmoteFlags, EmoteId};
use super::emote_set::EmoteSetId;
use super::role::RoleId;
use super::ticket::TicketId;
use super::user::connection::Platform;
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
	Ticket {
		target_id: TicketId,
		data: AuditLogTicketData,
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
	Login { platform: Platform },
	Logout,
	AddEditor { editor_id: UserId },
	RemoveEditor { editor_id: UserId },
	AddRole { role_id: RoleId },
	RemoveRole { role_id: RoleId },
	Ban,
	Unban,
	Merge,
	Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditLogTicketData {
	Create,
	AddMember { member: UserId },
	RemoveMember { member: UserId },
	ChangeOpen { old: bool, new: bool },
}
