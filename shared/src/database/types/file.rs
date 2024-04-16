use postgres_types::{FromSql, ToSql};
use crate::types::old::{ImageFile as OldImageFile, ImageFormat as OldImageFormat};

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
	Image {
		input: FileProperties<ImageFile>,
		pending: bool,
		outputs: Vec<FileProperties<ImageFile>>,
	},
	Other(FileProperties<()>),
}

impl FileSetProperties {
	pub fn as_image(&self) -> Option<&[FileProperties<ImageFile>]> {
		match self {
			Self::Image { outputs, .. } => Some(outputs),
			_ => None,
		}
	}

	pub fn pending(&self) -> bool {
		match self {
			Self::Image { pending, .. } => *pending,
			_ => false,
		}
	}

	pub fn default_image(&self) -> Option<&FileProperties<ImageFile>> {
		match self {
			Self::Image { outputs, .. } => outputs.iter().max_by_key(|image| image.extra.scale),
			_ => None,
		}
	}

	pub fn as_old_image_files(&self) -> Vec<OldImageFile> {
		match self {
			Self::Image { outputs, .. } => outputs
				.iter()
				.flat_map(|image| {
					image.extra.variants.iter().filter_map(|variant| {
						let format = match variant.format {
							ImageFormat::Webp => OldImageFormat::Webp,
							ImageFormat::Avif => OldImageFormat::Avif,
							_ => return None,
						};

						if image.extra.frame_count > 1 && variant.is_static {
							return None;
						}

						Some(OldImageFile {
							name: format!("{}x.{}", image.extra.scale, format.as_str()),
							static_name: format!("{}x_static.{}", image.extra.scale, format.as_str()),
							width: image.extra.width,
							height: image.extra.height,
							frame_count: image.extra.frame_count,
							size: image.size,
							format,
						})
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
	pub scale: u32,
	pub width: u32,
	pub height: u32,
	pub frame_count: u32,
	pub variants: Vec<ImageFileVariant>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ImageFileVariant {
	pub format: ImageFormat,
	pub is_static: bool,
	pub size: u64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FileProperties<E> {
	pub path: String,
	pub size: u64,
	pub mime: Option<String>,
	pub extra: E,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum ImageFormat {
	#[default]
	Webp,
	Avif,
	Gif,
	Png,
}
