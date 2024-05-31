use super::UserId;
use crate::database::{Collection, Id};

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
	// TODO
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
