use super::{Platform, UserId};
use crate::database::{Collection, Id};

pub type UserPresenceId = Id<UserPresence>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserPresence {
	#[serde(rename = "_id")]
	pub id: UserPresenceId,
	pub user_id: UserId,
	pub platform: Platform,
	pub platform_room_id: String,
	pub authentic: bool,
	pub ip_address: std::net::IpAddr,
	#[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for UserPresence {
	const COLLECTION_NAME: &'static str = "user_presences";
}
