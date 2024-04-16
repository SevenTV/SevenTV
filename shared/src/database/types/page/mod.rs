use crate::database::Table;

mod author;
mod file;

pub use author::*;
pub use file::*;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Page {
	pub id: ulid::Ulid,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	pub keywords: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub settings: PageSettings,
}

impl Table for Page {
	const TABLE_NAME: &'static str = "pages";
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum PageKind {
	#[default]
	Support,
	Blog,
	General,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct PageSettings {}
