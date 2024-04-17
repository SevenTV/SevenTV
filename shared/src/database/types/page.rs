use bson::oid::ObjectId;

use crate::database::Collection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	pub keywords: Vec<String>,
	pub author_ids: Vec<ObjectId>,
	pub file_ids: Vec<ObjectId>,
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
