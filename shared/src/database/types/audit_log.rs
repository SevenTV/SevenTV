use crate::database::Id;

use super::emote::EmoteId;
use super::emote_set::EmoteSetId;
use super::role::RoleId;
use super::ticket::TicketId;
use super::user::UserId;
use super::{Collection, GenericCollection};

pub type AuditLogId = Id<AuditLog>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditLog {
	#[serde(rename = "_id")]
	pub id: AuditLogId,
	pub actor_id: Option<UserId>,
	pub data: AuditLogData,
}

impl Collection for AuditLog {
	const COLLECTION_NAME: &'static str = "audit_logs";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"actor_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"data.target_id": 1,
				})
				.build(),
		]
	}
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<AuditLog>()]
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
	ChangeSettings {
		old: EmoteSetSettingsChange,
		new: EmoteSetSettingsChange,
	},
	AddEmote {
		emote_id: EmoteId,
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
	Register,
	Login,
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
	ChangeAssignees { added: Vec<UserId>, removed: Vec<UserId> },
	ChangeOpen { old: bool, new: bool },
	Delete,
}

// same as database::EmoteSettings but different
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EmoteSettingsChange {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub public_listed: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub private: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nsfw: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_zero_width: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub approved_personal: Option<bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct EmoteSetSettingsChange {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub capacity: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub privileged: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub immutable: Option<bool>,
}
