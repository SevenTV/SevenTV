use ulid::Ulid;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserBan {
	pub id: Ulid,
	pub user_id: Ulid,
	pub created_by_id: Option<Ulid>,
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
