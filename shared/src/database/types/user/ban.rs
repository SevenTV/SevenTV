use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserBan {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub user_id: ObjectId,
	pub created_by_id: Option<ObjectId>,
	pub reason: String,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for UserBan {
	const COLLECTION_NAME: &'static str = "user_bans";
}
