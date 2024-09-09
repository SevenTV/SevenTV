use macros::MongoCollection;
use serde::{Deserialize, Serialize};

use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::Id;

pub type SpecialEventId = Id<SpecialEvent>;

#[derive(Debug, Clone, Serialize, Deserialize, MongoCollection)]
#[mongo(collection_name = "special_events")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[mongo(search = "crate::typesense::types::product::special_event::SpecialEvent")]
#[serde(deny_unknown_fields)]
pub struct SpecialEvent {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SpecialEventId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<SpecialEvent>()]
}
