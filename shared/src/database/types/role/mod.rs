use self::permissions::Permissions;
use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

pub mod permissions;

pub type RoleId = Id<Role>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "roles")]
#[mongo(index(fields(rank = 1), unique))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::role::Role")]
#[serde(deny_unknown_fields)]
pub struct Role {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: RoleId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	pub permissions: Permissions,
	pub hoist: bool,
	pub color: Option<i32>,
	pub rank: i32,
	pub applied_rank: Option<i32>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Role>()]
}
