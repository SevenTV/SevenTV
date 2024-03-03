use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserEditor {
	pub user_id: ulid::Ulid,
	pub editor_id: ulid::Ulid,
	pub state: UserEditorState,
	#[from_row(from_fn = "scuffle_utils::database::json")]
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

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "user_editor_state")]
pub enum UserEditorState {
	#[default]
	#[postgres(name = "PENDING")]
	Pending,
	#[postgres(name = "ACCEPTED")]
	Accepted,
	#[postgres(name = "REJECTED")]
	Rejected,
}

impl Table for UserEditor {
	const TABLE_NAME: &'static str = "user_editors";
}
