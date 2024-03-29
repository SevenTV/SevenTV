use shared::types::old::ImageHostFormat;

use crate::database::{ImageFileData, Table};

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct EmoteFile {
	pub emote_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: ImageFileData,
}

impl Table for EmoteFile {
	const TABLE_NAME: &'static str = "emote_files";
}
