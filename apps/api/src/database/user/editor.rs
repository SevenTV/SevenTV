use postgres_types::{FromSql, ToSql};
use ulid::Ulid;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserEditor {
	pub user_id: Ulid,
	pub editor_id: Ulid,
	pub state: UserEditorState,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: UserEditorSettings,
	pub added_by_id: Option<Ulid>,
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
pub enum UserEditorState {
	#[default]
	#[postgres(name = "PENDING")]
	Pending,
	#[postgres(name = "ACCEPTED")]
	Accepted,
	#[postgres(name = "REJECTED")]
	Rejected,
}
