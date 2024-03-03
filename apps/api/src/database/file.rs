use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct File {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub kind: FileKind,
	pub path: String,
	pub mine_type: String,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub properties: FileProperties,
}

impl Table for File {
	const TABLE_NAME: &'static str = "files";
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "file_kind")]
pub enum FileKind {
	#[default]
	#[postgres(name = "OTHER")]
	Other,
	#[postgres(name = "PROFILE_PICTURE")]
	ProfilePicture,
	#[postgres(name = "BADGE")]
	Badge,
	#[postgres(name = "PAINT")]
	Paint,
	#[postgres(name = "EMOTE")]
	Emote,
	#[postgres(name = "PRODUCT")]
	Product,
	#[postgres(name = "PAGE")]
	Page,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct FileProperties {}
