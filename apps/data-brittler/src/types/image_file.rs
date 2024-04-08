use std::collections::HashMap;
use std::num::ParseIntError;

use once_cell::sync::OnceCell;
use shared::database::{FileProperties, ImageFormat};

// TODO: Default values for width, height, and frame_count
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

impl From<ImageFile> for FileProperties<shared::database::ImageFile> {
	fn from(value: ImageFile) -> Self {
		Self {
			path: format!("{}/{}", value.bucket, value.key),
			size: value.size,
			mime: Some(value.content_type),
			extra: shared::database::ImageFile {
				scale: 1,
				width: value.width,
				height: value.height,
				frame_count: value.frame_count,
				variants: vec![],
			},
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ImageFileError {
	#[error("invalid image file name: {0}")]
	InvalidImageFileName(String),
	#[error("invalid image file scale: {0}")]
	InvalidImageFileScale(String, #[source] ParseIntError),
	#[error("invalid image file content type: {0}")]
	InvalidImageFileContentType(String),
}

static IMAGE_FILE_REGEX: OnceCell<regex::Regex> = OnceCell::new();

pub fn image_files_to_file_properties<I: IntoIterator<Item = ImageFile>>(
	image_files: I,
) -> Result<Vec<FileProperties<shared::database::ImageFile>>, ImageFileError> {
	let mut files = HashMap::new();

	for old_file in image_files {
		let rexp = IMAGE_FILE_REGEX.get_or_init(|| regex::Regex::new(r"^(\d+)x(_static)?$").unwrap());
		let captures = rexp
			.captures(&old_file.name)
			.ok_or_else(|| ImageFileError::InvalidImageFileName(old_file.name.clone()))?;

		let scale = captures.get(1).unwrap().as_str();
		let scale = scale
			.parse::<u32>()
			.map_err(|e| ImageFileError::InvalidImageFileScale(scale.to_string(), e))?;

		let is_static = captures.get(2).is_some();

		let key: Vec<&str> = old_file.key.split_inclusive('/').collect();
		let key: String = key.into_iter().rev().skip(1).rev().collect();
		let path = format!("{}/{}", old_file.bucket, key);

		let new_file = files.entry(scale).or_insert_with(|| shared::database::FileProperties {
			path,
			size: old_file.size,
			mime: Some(old_file.content_type.clone()),
			extra: shared::database::ImageFile {
				scale,
				width: old_file.width,
				height: old_file.height,
				frame_count: old_file.frame_count,
				variants: vec![],
			},
		});
		new_file.extra.variants.push(shared::database::ImageFileVariant {
			format: match old_file.content_type.as_str() {
				"image/webp" => ImageFormat::Webp,
				"image/avif" => ImageFormat::Avif,
				"image/gif" => ImageFormat::Gif,
				"image/png" => ImageFormat::Png,
				_ => return Err(ImageFileError::InvalidImageFileContentType(old_file.content_type)),
			},
			is_static,
			size: old_file.size,
		});
	}

	Ok(files.into_values().collect())
}
