use super::image_set::ImageSet;
use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

pub type BadgeId = Id<Badge>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "badges")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[mongo(search = "crate::typesense::types::badge::Badge")]
#[serde(deny_unknown_fields)]
pub struct Badge {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: BadgeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub image_set: ImageSet,
	pub created_by_id: UserId,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Badge>()]
}
