use bson::oid::ObjectId;

use crate::database::Collection;
use crate::types::old::{UserEditorModel, UserEditorModelPermission};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserEditor {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub user_id: ObjectId,
	pub editor_id: ObjectId,
	pub state: UserEditorState,
	pub notes: String,
	pub permissions: UserEditorPermissions,
	pub added_by_id: Option<ObjectId>,
}

impl UserEditor {
	pub fn into_old_model(self) -> Option<UserEditorModel> {
		if self.state != UserEditorState::Accepted {
			return None;
		}

		Some(UserEditorModel {
			id: self.editor_id,
			added_at: self.id.timestamp().timestamp_millis(),
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
