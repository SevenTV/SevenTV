use clickhouse::Row;

use super::{EmoteId, RoleId, UserId};
use crate::database;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct EmoteActivity {
	#[serde(with = "clickhouse::serde::uuid")]
	pub emote_id: uuid::Uuid,
	#[serde(with = "clickhouse::serde::uuid::option")]
	pub actor_id: Option<uuid::Uuid>,
	pub kind: EmoteActivityKind,
	#[serde(with = "super::json_string::optional")]
	pub data: Option<EmoteActivityData>,
	#[serde(with = "clickhouse::serde::time::datetime64::millis")]
	pub timestamp: time::OffsetDateTime,
}

#[derive(Debug, Clone, Copy, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum EmoteActivityKind {
	Upload = 0,
	Process = 1,
	Edit = 2,
	Merge = 3,
	Delete = 4,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum EmoteActivityData {
	ChangeName {
		old: String,
		new: String,
	},
	Merge {
		new_emote_id: EmoteId,
	},
	ChangeOwner {
		old: UserId,
		new: UserId,
	},
	ChangeTags {
		new: Vec<String>,
		old: Vec<String>,
	},
	ChangeSettings {
		old: EmoteSettingsChange,
		new: EmoteSettingsChange,
	},
}

// same as database::EmoteSettings but different
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EmoteSettingsChange {
	pub public_listed: Option<bool>,
	pub private: Option<bool>,
	pub nsfw: Option<bool>,
	pub default_zero_width: Option<bool>,
	pub approved_personal: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct EmoteSetActivity {
	#[serde(with = "clickhouse::serde::uuid")]
	pub emote_set_id: uuid::Uuid,
	#[serde(with = "clickhouse::serde::uuid::option")]
	pub actor_id: Option<uuid::Uuid>,
	pub kind: EmoteSetActivityKind,
	#[serde(with = "super::json_string::optional")]
	pub data: Option<EmoteSetActivityData>,
	#[serde(with = "clickhouse::serde::time::datetime64::millis")]
	pub timestamp: time::OffsetDateTime,
}

#[derive(Debug, Clone, Copy, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum EmoteSetActivityKind {
	Create = 0,
	Edit = 1,
	Delete = 2,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum EmoteSetActivityData {
	ChangeName {
		old: String,
		new: String,
	},
	ChangeTags {
		added: Vec<String>,
		removed: Vec<String>,
	},
	ChangeSettings {
		old: EmoteSetSettingsChange,
		new: EmoteSetSettingsChange,
	},
	ChangeEmotes {
		added: Vec<EmoteId>,
		removed: Vec<EmoteId>,
	},
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct EmoteSetSettingsChange {
	pub capacity: Option<u32>,
	pub privileged: Option<bool>,
	pub immutable: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct UserActivity {
	#[serde(with = "clickhouse::serde::uuid")]
	pub user_id: uuid::Uuid,
	#[serde(with = "clickhouse::serde::uuid::option")]
	pub actor_id: Option<uuid::Uuid>,
	pub kind: UserActivityKind,
	#[serde(with = "super::json_string::optional")]
	pub data: Option<UserActivityData>,
	#[serde(with = "clickhouse::serde::time::datetime64::millis")]
	pub timestamp: time::OffsetDateTime,
}

#[derive(Debug, Clone, Copy, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum UserActivityKind {
	Register = 0,
	Login = 1,
	Logout = 2,
	Edit = 3,
	Delete = 4,
	Merge = 5,
	Ban = 6,
	Unban = 7,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum UserActivityData {
	ChangeEditors { added: Vec<UserId>, removed: Vec<UserId> },
	ChangeRoles { added: Vec<RoleId>, removed: Vec<RoleId> },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct TicketActivity {
	#[serde(with = "clickhouse::serde::uuid")]
	pub ticket_id: uuid::Uuid,
	#[serde(with = "clickhouse::serde::uuid::option")]
	pub actor_id: Option<uuid::Uuid>,
	pub kind: TicketActivityKind,
	#[serde(with = "super::json_string::optional")]
	pub data: Option<TicketActivityData>,
	#[serde(with = "clickhouse::serde::time::datetime64::millis")]
	pub timestamp: time::OffsetDateTime,
}

#[derive(Debug, Clone, Copy, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum TicketActivityKind {
	Create = 0,
	Edit = 1,
	Delete = 2,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum TicketActivityData {
	ChangeStatus {
		old: database::TicketStatus,
		new: database::TicketStatus,
	},
	ChangeAssignees {
		added: Vec<UserId>,
		removed: Vec<UserId>,
	},
}
