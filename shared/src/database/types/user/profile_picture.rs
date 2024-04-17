use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserProfilePicture {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub user_id: ObjectId,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for UserProfilePicture {
	const COLLECTION_NAME: &'static str = "user_profile_pictures";
}
