use super::{FileSetId, UserId};
use crate::database::{Collection, Id};

pub type PageId = Id<Page>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
	#[serde(rename = "_id", skip_serializing_if = "Id::is_nil")]
	pub id: PageId,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	pub keywords: Vec<String>,
	pub author_ids: Vec<UserId>,
	pub file_ids: Vec<FileSetId>,
}

impl Collection for Page {
	const COLLECTION_NAME: &'static str = "pages";
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum PageKind {
	#[default]
	Support,
	Blog,
	General,
}
