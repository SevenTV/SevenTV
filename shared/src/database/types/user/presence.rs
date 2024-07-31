use super::connection::Platform;
use super::UserId;
use crate::database::types::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "user_presences")]
#[mongo(index(fields(platform = 1, platform_room_id = 1, user_id = 1)))]
#[mongo(index(fields(expires_at = 1), expire_after = 0))]
#[serde(deny_unknown_fields)]
pub struct UserPresence {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: Id<UserPresence>,
	pub platform: Platform,
	pub platform_room_id: String,
	pub user_id: UserId,
	pub authentic: bool,

	pub ip_address: std::net::IpAddr,
	#[serde(with = "crate::database::serde")]
	pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<UserPresence>()]
}
