use ulid::Ulid;

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

impl ImageHostKind {
	pub fn create_base_url(&self, base: &str, id: Ulid) -> String {
		format!("{base}/{self}/{id}")
	}

	pub fn create_full_url(&self, base: &str, id: Ulid, scale: u32, format: ImageFormat) -> String {
		format!("{base}/{self}/{id}/{scale}x.{}", format.as_str())
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

impl ImageHost {
	pub fn new(base_url: &str, kind: ImageHostKind, id: Ulid, files: Vec<ImageFile>) -> Self {
		Self {
			url: kind.create_base_url(base_url, id),
			files,
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
