use derive_builder::Builder;

use super::image_set::ImageSet;
use super::user::UserId;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub type PageId = Id<Page>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct Page {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: PageId,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	#[builder(default)]
	pub keywords: Vec<String>,
	pub author_ids: Vec<UserId>,
	#[builder(default)]
	pub image_sets: Vec<ImageSet>,
}

impl Collection for Page {
	const COLLECTION_NAME: &'static str = "pages";
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum PageKind {
	Support = 0,
	Blog = 1,
	General = 2,
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Page>()]
}
