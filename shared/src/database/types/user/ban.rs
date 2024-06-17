use serde::Deserialize;

use super::ban_template::UserBanTemplateId;
use super::UserId;
use crate::database::role::permissions::Permissions;
use crate::database::Id;

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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserBan {
	pub id: UserBanId,
	pub created_by_id: UserId,
	pub reason: String,
	pub tags: Vec<String>,
	#[serde(
		serialize_with = "serialize_optional_datetime",
		deserialize_with = "deserialize_optional_datetime"
	)]
	pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub removed: Option<UserBanRemoved>,
	pub permissions: Permissions,
	pub template_id: Option<UserBanTemplateId>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserBanRemoved {
	pub removed_at: chrono::DateTime<chrono::Utc>,
	pub removed_by_id: UserId,
}
