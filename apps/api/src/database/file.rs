use postgres_types::{FromSql, ToSql};
use shared::types::old::{ImageHostFile, ImageHostFormat};

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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct ImageFileData {
	pub width: u32,
	pub height: u32,
	pub frame_count: Option<u32>,
	pub size: u64,
	pub format: ImageFileFormat,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
pub enum ImageFileFormat {
	#[default]
	Webp,
	Avif,
}

impl From<ImageFileFormat> for ImageHostFormat {
	fn from(value: ImageFileFormat) -> Self {
		match value {
			ImageFileFormat::Webp => Self::Webp,
			ImageFileFormat::Avif => Self::Avif,
		}
	}
}

impl ImageFileData {
	pub fn into_host_file(self, name: String) -> ImageHostFile {
		ImageHostFile {
			name: name.clone(),
			static_name: name,
			width: self.width,
			height: self.height,
			frame_count: self.frame_count.unwrap_or_default(),
			size: self.size,
			format: self.format.into(),
		}
	}
}
