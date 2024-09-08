use super::{MongoCollection, UserId};
use crate::database::duration::DurationUnit;
use crate::database::role::permissions::Permissions;
use crate::database::types::MongoGenericCollection;
use crate::database::Id;

pub type UserBanTemplateId = Id<UserBanTemplate>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "user_ban_templates")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[mongo(search = "crate::typesense::types::user::ban_template::UserBanTemplate")]
#[serde(deny_unknown_fields)]
pub struct UserBanTemplate {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: UserBanTemplateId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub pinned: bool,
	pub created_by: UserId,
	pub permissions: Permissions,
	pub duration: Option<DurationUnit>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<UserBanTemplate>()]
}
