use std::fmt::Display;
use std::str::FromStr;

use shared::database::badge::BadgeId;
use shared::database::emote::EmoteId;
use shared::database::paint::{PaintId, PaintLayerId};
use shared::database::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
	Badge {
		badge_id: BadgeId,
		file: ImageFile,
	},
	Emote {
		emote_id: EmoteId,
		file: ImageFile,
	},
	UserProfilePicture {
		user_id: UserId,
		avatar_id: String,
		file: ImageFile,
	},
	Paint {
		paint_id: PaintId,
		layer_id: PaintLayerId,
		file: ImageFile,
	},
	Misc {
		key: String,
	},
	Juicers,
}

impl Display for CacheKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Badge { badge_id, file } => write!(f, "badge/{badge_id}/{file}"),
			Self::Emote { emote_id, file } => write!(f, "emote/{emote_id}/{file}"),
			Self::Paint {
				paint_id,
				layer_id,
				file,
			} => write!(f, "paint/{paint_id}/layer/{layer_id}/{file}"),
			Self::UserProfilePicture {
				user_id,
				avatar_id,
				file,
			} => {
				write!(f, "user/{user_id}/profile-picture/{avatar_id}/{file}")
			}
			Self::Misc { key } => write!(f, "misc/{key}"),
			Self::Juicers => write!(f, "JUICERS.png"),
		}
	}
}

impl From<CacheKey> for String {
	fn from(value: CacheKey) -> Self {
		value.to_string()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageFile {
	pub name: ImageFileName,
	pub extension: ImageFileExtension,
	pub is_static: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageFileName {
	One,
	Two,
	Three,
	Four,
}

impl Display for ImageFileName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ImageFileName::One => write!(f, "1x"),
			ImageFileName::Two => write!(f, "2x"),
			ImageFileName::Three => write!(f, "3x"),
			ImageFileName::Four => write!(f, "4x"),
		}
	}
}

impl FromStr for ImageFileName {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"1x" => Ok(Self::One),
			"2x" => Ok(Self::Two),
			"3x" => Ok(Self::Three),
			"4x" => Ok(Self::Four),
			_ => Err("invalid image file name"),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageFileExtension {
	Avif,
	Gif,
	Png,
	Webp,
}

impl Display for ImageFileExtension {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ImageFileExtension::Avif => write!(f, "avif"),
			ImageFileExtension::Gif => write!(f, "gif"),
			ImageFileExtension::Png => write!(f, "png"),
			ImageFileExtension::Webp => write!(f, "webp"),
		}
	}
}

impl FromStr for ImageFileExtension {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"avif" => Ok(Self::Avif),
			"gif" => Ok(Self::Gif),
			"png" => Ok(Self::Png),
			"webp" => Ok(Self::Webp),
			_ => Err("invalid file extension"),
		}
	}
}

impl FromStr for ImageFile {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if let Some((mut name, ext)) = s.split_once('.') {
			let extension = ImageFileExtension::from_str(ext)?;

			let is_static = if let Some(n) = name.strip_suffix("_static") {
				name = n;
				true
			} else {
				false
			};

			Ok(Self {
				name: ImageFileName::from_str(name)?,
				extension,
				is_static,
			})
		} else {
			Ok(Self {
				name: ImageFileName::from_str(s)?,
				extension: ImageFileExtension::Webp,
				is_static: false,
			})
		}
	}
}

impl Display for ImageFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_static {
			write!(f, "{}_static.{}", self.name, self.extension)
		} else {
			write!(f, "{}.{}", self.name, self.extension)
		}
	}
}

impl serde::Serialize for ImageFile {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> serde::Deserialize<'de> for ImageFile {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		s.parse().map_err(serde::de::Error::custom)
	}
}
