use super::UserId;
use crate::database::types::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

pub type UserSessionId = Id<UserSession>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "user_sessions")]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(expires_at = 1), expire_after = 0))]
#[serde(deny_unknown_fields)]
pub struct UserSession {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserSessionId,
	pub user_id: UserId,
	#[serde(with = "crate::database::serde")]
	pub expires_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub last_used_at: chrono::DateTime<chrono::Utc>,
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<UserSession>()]
}
