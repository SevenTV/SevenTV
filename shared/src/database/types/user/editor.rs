use super::UserId;
use crate::database::{Collection, Id};
use crate::types::old::{UserEditorModel, UserEditorModelPermission};

pub type UserEditorId = Id<UserEditor>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserEditor {
	#[serde(rename = "_id", with = "crate::database::id::bson")]
	pub id: UserEditorId,
	pub user_id: UserId,
	pub editor_id: UserId,
	pub state: UserEditorState,
	pub notes: Option<String>,
	pub permissions: UserEditorPermissions,
	pub added_by_id: Option<UserId>,
}

impl UserEditor {
	pub fn into_old_model(self) -> Option<UserEditorModel> {
		if self.state != UserEditorState::Accepted {
			return None;
		}

		Some(UserEditorModel {
			id: self.editor_id,
			added_at: self.id.timestamp_ms(),
			permissions: UserEditorModelPermission::ModifyEmotes | UserEditorModelPermission::ManageEmoteSets,
			visible: true,
		})
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct UserEditorPermissions {
	// TODO
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum UserEditorState {
	#[default]
	Pending,
	Accepted,
	Rejected,
}

impl Collection for UserEditor {
	const COLLECTION_NAME: &'static str = "user_editors";
}
