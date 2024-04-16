use bson::oid::ObjectId;

use crate::database::Collection;

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
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct UserEditorPermissions {
	// TODO
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum UserEditorState {
	#[default]
	Pending,
	Accepted,
	Rejected,
}

impl Collection for UserEditor {
	const NAME: &'static str = "user_editors";
}
