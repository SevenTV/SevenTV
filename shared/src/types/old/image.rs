use crate::database::{Id, Image, ImageSet};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/model.go#L47
pub struct ImageHost {
	pub url: String,
	pub files: Vec<ImageFile>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ImageHostKind {
	Badge,
	Emote,
	Paint,
	ProfilePicture,
}

impl ImageHost {
	pub fn from_image_set<T>(image_set: &ImageSet, cdn_base_url: &str, kind: ImageHostKind, id: &Id<T>) -> Self {
		Self {
			url: kind.create_base_url(cdn_base_url, id),
			files: image_set.outputs.iter().map(Into::into).collect(),
		}
	}
}

impl ImageHostKind {
	pub fn create_base_url<T>(&self, base: &str, id: &Id<T>) -> String {
		format!("{base}/{self}/{id}")
	}
}

impl std::fmt::Display for ImageHostKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Badge => write!(f, "badge"),
			Self::Emote => write!(f, "emote"),
			Self::ProfilePicture => write!(f, "profile-picture"),
			Self::Paint => write!(f, "paint"),
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/model.go#L52
pub struct ImageFile {
	pub name: String,
	pub static_name: String,
	pub width: u32,
	pub height: u32,
	pub frame_count: u32,
	pub size: u64,
	pub format: ImageFormat,
}

impl From<&Image> for ImageFile {
	fn from(value: &Image) -> Self {
		Self {
			name: value.path.clone(),
			static_name: value.path.clone(),
			width: value.width,
			height: value.height,
			frame_count: value.frame_count,
			size: value.size,
			format: match &value.mime {
				mime if mime.starts_with("image/webp") => ImageFormat::Webp,
				mime if mime.starts_with("image/avif") => ImageFormat::Avif,
				_ => ImageFormat::Webp,
			},
		}
	}
}

impl From<Image> for ImageFile {
	fn from(value: Image) -> Self {
		(&value).into()
	}
}

#[derive(Debug, Copy, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/model.go#L63
pub enum ImageFormat {
	#[default]
	Webp,
	Avif,
}

impl ImageFormat {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Webp => "webp",
			Self::Avif => "avif",
		}
	}
}
