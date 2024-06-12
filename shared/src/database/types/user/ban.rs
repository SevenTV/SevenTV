use serde::Deserialize;

use super::UserId;
use crate::database::{Collection, Id, UserBanRoleId};

pub type UserBanId = Id<UserBan>;

fn serialize_optional_datetime<S>(data: &Option<chrono::DateTime<chrono::Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	match data {
		Some(data) => mongodb::bson::serde_helpers::serialize_chrono_datetime_as_bson_datetime(data, serializer),
		None => serializer.serialize_none(),
	}
}

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	match Option::<mongodb::bson::DateTime>::deserialize(deserializer)? {
		Some(data) => Ok(Some(data.into())),
		None => Ok(None),
	}
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserBan {
	#[serde(rename = "_id")]
	pub id: UserBanId,
	pub user_id: UserId,
	pub created_by_id: Option<UserId>,
	pub reason: String,
	#[serde(
		serialize_with = "serialize_optional_datetime",
		deserialize_with = "deserialize_optional_datetime"
	)]
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub role_id: UserBanRoleId,
}

impl Collection for UserBan {
	const COLLECTION_NAME: &'static str = "user_bans";
}
