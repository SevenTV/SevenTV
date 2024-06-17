use super::image_set::ImageSet;
use super::user::UserId;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub type PageId = Id<Page>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
	#[serde(rename = "_id")]
	pub id: PageId,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	pub keywords: Vec<String>,
	pub author_ids: Vec<UserId>,
	pub image_sets: Vec<ImageSet>,
}

impl Collection for Page {
	const COLLECTION_NAME: &'static str = "pages";
}

#[derive(Debug, Clone, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum PageKind {
	#[default]
	Support = 0,
	Blog = 1,
	General = 2,
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Page>()]
}
