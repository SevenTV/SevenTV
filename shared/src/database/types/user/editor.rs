use bitmask_enum::bitmask;

use super::UserId;
use crate::database::{AllowDeny, BitMask, Collection, EmoteSetPermission, Id};

pub type UserEditorId = Id<UserEditor>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserEditor {
	#[serde(rename = "_id")]
	pub id: UserEditorId,
	pub user_id: UserId,
	pub editor_id: UserId,
	pub state: UserEditorState,
	pub notes: Option<String>,
	pub permissions: UserEditorPermissions,
	pub added_by_id: Option<UserId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct UserEditorPermissions {
	#[serde(skip_serializing_if = "AllowDeny::is_empty")]
	#[serde(default)]
	pub emote_set: AllowDeny<EmoteSetPermission>,
}

impl UserEditorPermissions {
	pub fn has_emote_set(&self, permission: EmoteSetPermission) -> bool {
		self.emote_set.permission().contains(permission)
			|| self.emote_set.permission().contains(EmoteSetPermission::Admin)
	}
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum UserEditorState {
	#[default]
	Pending = 0,
	Accepted = 1,
	Rejected = 2,
}

impl Collection for UserEditor {
	const COLLECTION_NAME: &'static str = "user_editors";
}
