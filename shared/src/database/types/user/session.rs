use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserSession {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub user_id: ObjectId,
	pub expires_at: chrono::DateTime<chrono::Utc>,
	pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for UserSession {
	const COLLECTION_NAME: &'static str = "user_sessions";
}
