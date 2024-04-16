use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserEditor {
	pub user_id: ulid::Ulid,
	pub editor_id: ulid::Ulid,
	pub state: UserEditorState,
	pub data: UserEditorSettings,
	pub added_by_id: Option<ulid::Ulid>,
	pub added_at: chrono::DateTime<chrono::Utc>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserEditorSettings {
	pub notes: String,
	pub permissions: UserEditorPermissions,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserEditorPermissions {}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum UserEditorState {
	#[default]
	Pending,
	Accepted,
	Rejected,
}

impl Table for UserEditor {
	const TABLE_NAME: &'static str = "user_editors";
}
