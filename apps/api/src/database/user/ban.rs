use ulid::Ulid;

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserBan {
	pub id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub created_by_id: Option<ulid::Ulid>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: UserBanData,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserBanData {
	pub reason: String,
}

impl Table for UserBan {
	const TABLE_NAME: &'static str = "user_bans";
}
