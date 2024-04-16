use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct PageAuthor {
	pub page_id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub order: i16,
}

impl Table for PageAuthor {
	const TABLE_NAME: &'static str = "page_authors";
}
