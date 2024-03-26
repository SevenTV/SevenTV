use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct PageFile {
	pub page_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
}

impl Table for PageFile {
	const TABLE_NAME: &'static str = "page_files";
}
