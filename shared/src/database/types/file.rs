use crate::database::{Collection, Id};
use crate::types::old::{ImageFile as OldImageFile, ImageFormat as OldImageFormat};

use super::{BadgeId, EmoteId, PageId, PaintId, ProductId, TicketId, UserId};

pub type FileSetId = Id<FileSet>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileSet {
	#[serde(rename = "_id")]
	pub id: FileSetId,
	pub ref_id: FileSetRefId,
	pub kind: FileSetKind,
	pub data: FileSetData,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum FileSetRefId {
	Ticket(TicketId),
	ProfilePicture(UserId),
	Badge(BadgeId),
	Paint(PaintId),
	Emote(EmoteId),
	Product(ProductId),
	Page(PageId),
}

impl Collection for FileSet {
	const COLLECTION_NAME: &'static str = "file_sets";
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum FileSetData {
	ImageProcessor(ImageProcessorResult),
	PendingImageProcessor(GenericFile),
	Generic(GenericFile),
}

impl FileSetData {
	pub fn as_generic(&self) -> Option<&GenericFile> {
		match self {
			Self::Generic(g) => Some(g),
			_ => None,
		}
	}

	pub fn as_image_processor(&self) -> Option<&ImageProcessorResult> {
		match self {
			Self::ImageProcessor(i) => Some(i),
			_ => None,
		}
	}

	pub fn as_pending_image_processor(&self) -> Option<&GenericFile> {
		match self {
			Self::PendingImageProcessor(i) => Some(i),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GenericFile {
	pub path: String,
	pub mime: String,
	pub size: u64,
	pub sha256: String,
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
	#[serde(flatten)]
    pub generic: GenericFile,
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
	pub scale: Option<u32>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImageProcessorResult {
    pub input: Image,
    pub outputs: Vec<Image>,
}

impl ImageProcessorResult {
	pub fn as_old_image_files(&self) -> Vec<OldImageFile> {
		self.outputs.iter().map(|output| OldImageFile {
			name: output.generic.path.clone(),
			static_name: output.generic.path.clone(),
			width: output.width,
			height: output.height,
			frame_count: output.frame_count,
			size: output.generic.size,
			format: OldImageFormat::from_mime(&output.generic.mime),
		}).collect()
	}
}