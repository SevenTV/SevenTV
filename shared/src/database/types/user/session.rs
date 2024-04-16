use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserSession {
	pub id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub expires_at: chrono::DateTime<chrono::Utc>,
	pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl Table for UserSession {
	const TABLE_NAME: &'static str = "user_sessions";
}
