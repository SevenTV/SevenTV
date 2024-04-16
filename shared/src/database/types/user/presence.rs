use super::Platform;
use crate::database::Table;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserPresence {
	pub user_id: ulid::Ulid,
	pub platform: Platform,
	pub platform_room_id: String,
	pub authentic: bool,
	pub ip_address: std::net::IpAddr,
	pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

impl Table for UserPresence {
	const TABLE_NAME: &'static str = "user_presences";
}
