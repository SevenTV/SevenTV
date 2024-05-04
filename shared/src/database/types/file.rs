use crate::database::{Collection, Id};
use crate::types::old::{ImageFile as OldImageFile, ImageFormat as OldImageFormat};

pub type FileSetId = Id<FileSet>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileSet {
	#[serde(rename = "_id")]
	pub id: FileSetId,
	pub kind: FileSetKind,
	pub authenticated: bool,
	pub properties: FileSetProperties,
}

impl Collection for FileSet {
	const COLLECTION_NAME: &'static str = "file_sets";
}

#[derive(Debug, Clone, Eq, PartialEq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum FileSetKind {
	Ticket = 0,
	ProfilePicture = 1,
	Badge = 2,
	Paint = 3,
	Emote = 4,
	Product = 5,
	Page = 6,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "data")]
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
#[serde(deny_unknown_fields)]
pub struct ImageFile {
	pub scale: u32,
	pub width: u32,
	pub height: u32,
	pub frame_count: u32,
	pub variants: Vec<ImageFileVariant>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageFileVariant {
	pub format: ImageFormat,
	pub is_static: bool,
	pub size: u64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileProperties<E> {
	pub path: String,
	pub size: u64,
	pub mime: Option<String>,
	pub extra: E,
}

#[derive(
	Debug, Copy, Clone, Default, PartialEq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, utoipa::ToSchema,
)]
#[repr(u8)]
pub enum ImageFormat {
	#[default]
	Webp = 0,
	Avif = 1,
	Gif = 2,
	Png = 3,
}
