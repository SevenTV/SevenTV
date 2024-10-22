use crate::database::image_set::{Image, ImageSet};

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

impl ImageHost {
	pub fn from_image_set_with_options(image_set: &ImageSet, cdn_base_url: &url::Url, webp_only: bool, without_protocol: bool) -> Self {
		let url = image_set.outputs.first().and_then(|i| {
			let path = i.path.clone();
			// keep everything until last '/'
			let split = path.split('/').collect::<Vec<_>>();
			cdn_base_url
				.join(&split.split_last()?.1.join("/"))
				.map(|u| {
					if !without_protocol {
						return u.to_string();
					}

					u.as_str()
						.trim_start_matches("https:")
						.trim_start_matches("http:")
						.to_string()
				})
				.ok()
		});

		let animated = image_set.outputs.iter().any(|i| i.frame_count > 1);

		let mut files: Vec<ImageFile> = image_set
			.outputs
			.iter()
			.filter(|i| (i.frame_count > 1) == animated)
			// Filter out any images with formats that should not be returned by the v3 api
			.filter(|i| {
				let Some(format) = ImageFormat::from_mime(&i.mime) else {
					return false;
				};

				!webp_only || format == ImageFormat::Webp
			})
			.map(Into::into)
			.collect();

		// sort by format, then name (scale)
		files.sort_by(|a, b| a.format.cmp(&b.format).then_with(|| a.name.cmp(&b.name)));

		Self {
			url: url.unwrap_or_default(),
			files,
		}
	}

	pub fn from_image_set(image_set: &ImageSet, cdn_base_url: &url::Url) -> Self {
		Self::from_image_set_with_options(image_set, cdn_base_url, false, true)
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
		// trim everything until last '/'
		let name = value.path.split('/').last().unwrap_or(&value.path).to_string();
		let (name, ext) = name.split_once('.').unwrap_or((&name, ""));

		Self {
			static_name: format!("{}_static.{}", name, ext),
			name: format!("{}.{}", name, ext),
			width: value.width as u32,
			height: value.height as u32,
			frame_count: value.frame_count as u32,
			size: value.size as u64,
			format: ImageFormat::from_mime(&value.mime).unwrap_or(ImageFormat::Webp),
		}
	}
}

impl From<Image> for ImageFile {
	fn from(value: Image) -> Self {
		(&value).into()
	}
}

#[derive(
	Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
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
