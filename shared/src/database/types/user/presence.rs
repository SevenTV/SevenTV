use bson::oid::ObjectId;

use super::Platform;
use crate::database::Collection;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserPresence {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub user_id: ObjectId,
	pub platform: Platform,
	pub platform_room_id: String,
	pub authentic: bool,
	pub ip_address: std::net::IpAddr,
	pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for UserPresence {
	const COLLECTION_NAME: &'static str = "user_presences";
}
