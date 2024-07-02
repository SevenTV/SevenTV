use super::TypesenseGenericCollection;
use crate::database::page::{PageId, PageKind};
use crate::database::user::UserId;
use crate::typesense::types::TypesenseCollection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "pages")]
#[serde(deny_unknown_fields)]
pub struct Page {
	pub id: PageId,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	pub keywords: Vec<String>,
	pub tags: Vec<String>,
	pub author_ids: Vec<UserId>,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<crate::database::page::Page> for Page {
	fn from(value: crate::database::page::Page) -> Self {
		Self {
			id: value.id,
			kind: value.kind,
			title: value.title,
			slug: value.slug,
			content_md: value.content_md,
			keywords: value.keywords,
			tags: value.tags,
			author_ids: value.author_ids,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: chrono::Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Page>()]
}
