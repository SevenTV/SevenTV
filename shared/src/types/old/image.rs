use crate::database::{Id, Image, ImageSet};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[graphql(complex, rename_fields = "snake_case")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/model.go#L47
pub struct ImageHost {
	pub url: String,
	#[graphql(skip)]
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
			files: image_set
				.outputs
				.iter()
				// Filter out any images with formats that should not be returned by the v3 api
				.filter(|i| ImageFormat::from_mime(&i.mime).is_some())
				.map(Into::into)
				.collect(),
		}
	}
}

#[async_graphql::ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl ImageHost {
	async fn files(&self, formats: Option<Vec<ImageFormat>>) -> Vec<ImageFile> {
		let formats = formats.unwrap_or_default();

		self.files
			.iter()
			.filter(|i| formats.is_empty() || formats.iter().any(|f| *f == i.format))
			.cloned()
			.collect()
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[graphql(name = "Image", rename_fields = "snake_case")]
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
		let name = value.path.clone();
		// trim everything until last '/'
		let name = name.split('/').last().unwrap_or(&name).to_string();

		Self {
			static_name: name.replace("x.", "x_static."),
			name,
			width: value.width,
			height: value.height,
			frame_count: value.frame_count,
			size: value.size,
			format: ImageFormat::from_mime(&value.mime).unwrap_or(ImageFormat::Webp),
		}
	}
}

impl From<Image> for ImageFile {
	fn from(value: Image) -> Self {
		(&value).into()
	}
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/model.go#L63
pub enum ImageFormat {
	#[default]
	Webp,
	Avif,
	Gif,
	Png,
}

async_graphql::scalar!(ImageFormat);

impl ImageFormat {
	pub fn from_mime(mime: &str) -> Option<Self> {
		match mime {
			mime if mime.starts_with("image/webp") => Some(Self::Webp),
			mime if mime.starts_with("image/avif") => Some(Self::Avif),
			mime if mime.starts_with("image/gif") => Some(Self::Gif),
			mime if mime.starts_with("image/png") => Some(Self::Png),
			_ => None,
		}
	}

	pub fn to_mime(&self) -> &'static str {
		match self {
			Self::Webp => "image/webp",
			Self::Avif => "image/avif",
			Self::Gif => "image/gif",
			Self::Png => "image/png",
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Webp => "webp",
			Self::Avif => "avif",
			Self::Gif => "gif",
			Self::Png => "png",
		}
	}
}
