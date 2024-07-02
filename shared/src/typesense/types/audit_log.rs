use chrono::Utc;

use super::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};
use crate::database::audit_log::{
	AuditLogData, AuditLogEmoteData, AuditLogEmoteSetData, AuditLogId, AuditLogTicketData, AuditLogUserData,
};
use crate::database::user::UserId;
use crate::database::{self, Id};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "audit_logs")]
#[serde(deny_unknown_fields)]
pub struct AuditLog {
	pub id: AuditLogId,
	pub actor_id: Option<UserId>,
	pub target_id: Id<()>,
	pub kind: TargetKind,
	pub action: ActionKind,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::audit_log::AuditLog> for AuditLog {
	fn from(value: database::audit_log::AuditLog) -> Self {
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
	Ticket = 3,
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

	UserLogin = 5,
	UserLogout = 6,
	UserBan = 7,
	UserUnban = 8,
}

impl_typesense_type!(ActionKind, Int32);

fn split_kinds(data: AuditLogData) -> (Id<()>, TargetKind, ActionKind) {
	match data {
		AuditLogData::Emote { target_id, data } => (
			target_id.cast(),
			TargetKind::Emote,
			match data {
				AuditLogEmoteData::ChangeFlags { .. }
				| AuditLogEmoteData::ChangeName { .. }
				| AuditLogEmoteData::ChangeTags { .. }
				| AuditLogEmoteData::ChangeOwner { .. } => ActionKind::Modify,
				AuditLogEmoteData::Delete { .. } => ActionKind::Delete,
				AuditLogEmoteData::Process { .. } => ActionKind::EmoteProcess,
				AuditLogEmoteData::Upload { .. } => ActionKind::Create,
				AuditLogEmoteData::Merge { .. } => ActionKind::Merge,
			},
		),
		AuditLogData::EmoteSet { target_id, data } => (
			target_id.cast(),
			TargetKind::EmoteSet,
			match data {
				AuditLogEmoteSetData::ChangeCapacity { .. }
				| AuditLogEmoteSetData::ChangeName { .. }
				| AuditLogEmoteSetData::ChangeTags { .. }
				| AuditLogEmoteSetData::AddEmote { .. }
				| AuditLogEmoteSetData::RemoveEmote { .. }
				| AuditLogEmoteSetData::RenameEmote { .. } => ActionKind::Modify,
				AuditLogEmoteSetData::Create { .. } => ActionKind::Create,
				AuditLogEmoteSetData::Delete { .. } => ActionKind::Delete,
			},
		),
		AuditLogData::User { target_id, data } => (
			target_id.cast(),
			TargetKind::User,
			match data {
				AuditLogUserData::Ban => ActionKind::UserBan,
				AuditLogUserData::Unban => ActionKind::UserUnban,
				AuditLogUserData::Login { .. } => ActionKind::UserLogin,
				AuditLogUserData::Logout => ActionKind::UserLogout,
				AuditLogUserData::AddEditor { .. }
				| AuditLogUserData::RemoveEditor { .. }
				| AuditLogUserData::AddRole { .. }
				| AuditLogUserData::RemoveRole { .. } => ActionKind::Modify,
				AuditLogUserData::Delete { .. } => ActionKind::Delete,
				AuditLogUserData::Merge { .. } => ActionKind::Merge,
			},
		),
		AuditLogData::Ticket { target_id, data } => (
			target_id.cast(),
			TargetKind::Ticket,
			match data {
				AuditLogTicketData::AddMember { .. }
				| AuditLogTicketData::ChangeOpen { .. }
				| AuditLogTicketData::RemoveMember { .. } => ActionKind::Modify,
				AuditLogTicketData::Create { .. } => ActionKind::Create,
			},
		),
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<AuditLog>()]
}
