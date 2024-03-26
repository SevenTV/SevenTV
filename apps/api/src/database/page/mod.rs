use postgres_types::{FromSql, ToSql};

use crate::database::Table;

mod author;
mod file;

pub use author::*;
pub use file::*;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Page {
	pub id: ulid::Ulid,
	pub kind: PageKind,
	pub title: String,
	pub slug: String,
	pub content_md: String,
	pub keywords: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: PageSettings,
}

impl Table for Page {
	const TABLE_NAME: &'static str = "pages";
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "page_kind")]
pub enum PageKind {
	#[default]
	#[postgres(name = "SUPPORT")]
	Support,
	#[postgres(name = "BLOG")]
	Blog,
	#[postgres(name = "GENERAL")]
	General,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct PageSettings {}
