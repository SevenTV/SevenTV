use super::image_set::ImageSet;
use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};
use crate::typesense::types::impl_typesense_type;

pub type PageId = Id<Page>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "pages")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::page::Page")]
#[serde(deny_unknown_fields)]
pub struct Page {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: PageId,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	pub keywords: Vec<String>,
	pub tags: Vec<String>,
	pub author_ids: Vec<UserId>,
	pub image_sets: Vec<ImageSet>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum PageKind {
	#[default]
	Support = 0,
	Blog = 1,
	General = 2,
}

impl_typesense_type!(PageKind, Int32);

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Page>()]
}
