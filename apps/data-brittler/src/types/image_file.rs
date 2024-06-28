use shared::database::image_set::Image;

#[derive(Debug, serde::Deserialize)]
pub struct ImageFile {
	pub name: String,
	#[serde(default)]
	pub width: u32,
	#[serde(default)]
	pub height: u32,
	#[serde(default)]
	pub frame_count: u32,
	pub size: u64,
	pub content_type: String,
	pub key: String,
	pub bucket: String,
}

impl From<ImageFile> for Image {
	fn from(value: ImageFile) -> Self {
		Self {
			path: value.key,
			mime: value.content_type,
			size: value.size,
			width: value.width,
			height: value.height,
			frame_count: value.frame_count,
		}
	}
}
