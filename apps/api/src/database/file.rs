use postgres_types::{FromSql, ToSql};
use shared::types::old::{ImageHostFile, ImageHostFormat};

use crate::database::Table;

#[derive(Debug, Clone, postgres_from_row::FromRow)]
pub struct FileSet {
	pub id: ulid::Ulid,
	pub kind: FileSetKind,
	pub authenticated: bool,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub properties: FileSetProperties,
}

impl Table for FileSet {
	const TABLE_NAME: &'static str = "file_sets";
}

#[derive(Debug, Clone, Eq, PartialEq, ToSql, FromSql)]
#[postgres(name = "file_set_kind")]
pub enum FileSetKind {
	#[postgres(name = "TICKET")]
	Ticket,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FileSetProperties {
	Image(Vec<FileProperties<ImageFile>>),
	Other(FileProperties<()>),
}

impl FileSetProperties {
	pub fn as_image(&self) -> Option<&[FileProperties<ImageFile>]> {
		match self {
			Self::Image(images) => Some(images),
			_ => None,
		}
	}

	pub fn default_image(&self) -> Option<&FileProperties<ImageFile>> {
		match self {
			Self::Image(images) => images.iter().find(|image| image.extra.default),
			_ => None,
		}
	}

	pub fn as_old_image_files(&self, show_static: bool) -> Vec<ImageHostFile> {
		match self {
			Self::Image(images) => images
				.iter()
				.filter_map(|image| {
					if !matches!(image.mime, FileMime::Webp | FileMime::Avif)
						|| (image.extra.frame_count == 1 && !show_static)
					{
						return None;
					}

					Some(ImageHostFile {
						name: format!("{}x.{}", image.extra.scale, image.mime.extension()?),
						static_name: format!("{}x_static.{}", image.extra.scale, image.mime.extension()?),
						width: image.extra.width,
						height: image.extra.height,
						frame_count: image.extra.frame_count,
						size: image.size,
						format: image.mime.as_old_file()?,
					})
				})
				.collect(),
			_ => vec![],
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ImageFile {
	pub byte_size: u64,
	pub scale: u32,
	pub width: u32,
	pub height: u32,
	pub frame_count: u32,
	pub default: bool,
}

#[derive(Debug, Clone)]
pub enum FileMime {
	Webp,
	Avif,
	Gif,
	Png,
	Other(String),
}

impl FileMime {
	pub fn from_mime(mime: &str) -> Self {
		match mime {
			"image/webp" => Self::Webp,
			"image/avif" => Self::Avif,
			"image/gif" => Self::Gif,
			"image/png" => Self::Png,
			_ => Self::Other(mime.to_string()),
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Webp => "image/webp",
			Self::Avif => "image/avif",
			Self::Gif => "image/gif",
			Self::Png => "image/png",
			Self::Other(mime) => mime,
		}
	}

	pub fn extension(&self) -> Option<&str> {
		match self {
			Self::Webp => Some("webp"),
			Self::Avif => Some("avif"),
			Self::Gif => Some("gif"),
			Self::Png => Some("png"),
			Self::Other(mime) => None,
		}
	}

	pub fn as_old_file(&self) -> Option<ImageHostFormat> {
		match self {
			Self::Webp => Some(ImageHostFormat::Webp),
			Self::Avif => Some(ImageHostFormat::Avif),
			Self::Gif => Some(ImageHostFormat::Gif),
			Self::Png => Some(ImageHostFormat::Png),
			Self::Other(_) => None,
		}
	}
}

impl std::fmt::Display for FileMime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl Default for FileMime {
	fn default() -> Self {
		Self::Other("application/octet-stream".to_string())
	}
}

impl serde::Serialize for FileMime {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.as_str())
	}
}

impl<'de> serde::Deserialize<'de> for FileMime {
	fn deserialize<D>(deserializer: D) -> Result<FileMime, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(FileMime::from_mime(&s))
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FileProperties<E> {
	pub path: String,
	pub size: u64,
	pub mime: FileMime,
	pub extra: E,
}
