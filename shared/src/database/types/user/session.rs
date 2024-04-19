use super::UserId;
use crate::database::{Collection, Id};

pub type UserSessionId = Id<UserSession>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserSession {
	#[serde(rename = "_id", with = "crate::database::id::bson")]
	pub id: UserSessionId,
	pub user_id: UserId,
	pub expires_at: chrono::DateTime<chrono::Utc>,
	pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for UserSession {
	const COLLECTION_NAME: &'static str = "user_sessions";
}
