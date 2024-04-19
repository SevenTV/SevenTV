use super::UserId;
use crate::database::{Collection, Id};

pub type UserBanId = Id<UserBan>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserBan {
	#[serde(rename = "_id", skip_serializing_if = "Id::is_nil")]
	pub id: UserBanId,
	pub user_id: UserId,
	pub created_by_id: Option<UserId>,
	pub reason: String,
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Collection for UserBan {
	const COLLECTION_NAME: &'static str = "user_bans";
}
