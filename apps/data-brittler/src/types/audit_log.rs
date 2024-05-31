use mongodb::bson::oid::ObjectId;
use shared::old_types::EmoteFlagsModel;

use crate::types;

#[derive(Debug, serde::Deserialize)]
pub struct AuditLog {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub kind: AuditLogKind,
	pub actor_id: ObjectId,
	pub target_id: ObjectId,
	pub target_kind: AuditLogTargetKind,
	pub reason: Option<String>,
	#[serde(default, deserialize_with = "super::null_to_default")]
	pub changes: Vec<AuditLogChange>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EmoteVersionStateChange {
	pub listed: Option<bool>,
	pub allow_personal: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "key", content = "value")]
pub enum AuditLogChange {
	#[serde(rename = "name")]
	Name(AuditLogChangeSingleValue<String>),

	#[serde(rename = "capacity")]
	EmoteSetCapacity(AuditLogChangeSingleValue<i32>),
	#[serde(rename = "emotes")]
	EmoteSetEmotes(AuditLogChangeArray<types::EmoteSetEmote>),

	#[serde(rename = "versions")]
	EmoteVersions(AuditLogChangeArray<AuditLogChangeSingleValue<EmoteVersionStateChange>>),
	#[serde(rename = "new_emote_id")]
	NewEmoteId(AuditLogChangeSingleValue<ObjectIdWrapper>),
	#[serde(rename = "tags")]
	Tags(AuditLogChangeSingleValue<Vec<String>>),
	#[serde(rename = "flags")]
	Flags(AuditLogChangeSingleValue<EmoteFlagsModel>),
	#[serde(rename = "owner_id")]
	Owner(AuditLogChangeSingleValue<ObjectIdWrapper>),

	#[serde(rename = "editors")]
	UserEditors(AuditLogChangeArray<types::UserEditor>),
	#[serde(rename = "role_ids")]
	UserRoles(AuditLogChangeArray<Option<ObjectId>>),

	#[serde(rename = "status")]
	ReportStatus(AuditLogChangeSingleValue<types::ReportStatus>),
	#[serde(rename = "assignee_ids")]
	ReportAssignees(AuditLogChangeArray<ObjectId>),
}

#[derive(Debug, serde::Deserialize)]
pub struct AuditLogChangeSingleValue<T> {
	#[serde(rename = "n")]
	pub new: T,
	#[serde(rename = "o")]
	pub old: T,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum ObjectIdWrapper {
	Oid(ObjectId),
	InvalidOid(String),
}

impl ObjectIdWrapper {
	pub fn into_inner(self) -> Option<ObjectId> {
		match self {
			ObjectIdWrapper::Oid(oid) => Some(oid),
			ObjectIdWrapper::InvalidOid(_) => None,
		}
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct AuditLogChangeArray<T> {
	#[serde(default = "Vec::new")]
	pub added: Vec<T>,
	#[serde(default = "Vec::new")]
	pub removed: Vec<T>,
	#[serde(default = "Vec::new")]
	pub updated: Vec<T>,
}

// https://github.com/SevenTV/Common/blob/master/structures/v3/type.audit.go#L21
#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(u32)]
pub enum AuditLogKind {
	CreateEmote = 1,
	DeleteEmote = 2,
	DisableEmote = 3,
	UpdateEmote = 4,
	MergeEmote = 5,
	UndoDeleteEmote = 6,
	EnableEmote = 7,
	ProcessEmote = 8,

	SignUserToken = 20,
	SignCsrfToken = 21,
	RejectedAccess = 26,

	CreateUser = 30,
	DeleteUser = 31,
	BanUser = 32,
	EditUser = 33,
	UnbanUser = 36,

	CreateEmoteSet = 70,
	UpdateEmoteSet = 71,
	DeleteEmoteSet = 72,

	CreateReport = 80,
	UpdateReport = 81,

	ReadMessage = 90,
}

// https://github.com/SevenTV/Common/blob/master/structures/v3/structures.go#L97
#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(u32)]
pub enum AuditLogTargetKind {
	User = 1,
	Emote = 2,
	EmoteSet = 3,
	Role = 4,
	Entitlement = 5,
	Ban = 6,
	Message = 7,
	Report = 8,
	Presence = 9,
	Cosmetic = 10,
}
