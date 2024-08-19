use std::{fmt::Display, str::FromStr};

use shared::database::{badge::BadgeId, emote::EmoteId, user::UserId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
	Badge {
		id: BadgeId,
		file: BadgeFile,
	},
	Emote {
		id: EmoteId,
		file: ImageFile,
	},
	UserProfilePicture {
		user: UserId,
		avatar_id: String,
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
			Self::Badge { id, file } => write!(f, "badge/{id}/{file}"),
			Self::Emote { id, file } => write!(f, "emote/{id}/{file}"),
			Self::UserProfilePicture { user, avatar_id, file } => {
				write!(f, "user/{user}/{avatar_id}/{file}")
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
pub enum ImageFile {
	Image {
		name: ImageFileName,
		extension: ImageFileExtension,
		is_static: bool,
	},
	Archive,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BadgeFile {
	One,
	Two,
	Three,
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
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"1x" => Ok(Self::One),
			"2x" => Ok(Self::Two),
			"3x" => Ok(Self::Three),
			"4x" => Ok(Self::Four),
			_ => anyhow::bail!("invalid image file name"),
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
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"avif" => Ok(Self::Avif),
			"gif" => Ok(Self::Gif),
			"png" => Ok(Self::Png),
			"webp" => Ok(Self::Webp),
			_ => anyhow::bail!("invalid file extension"),
		}
	}
}

impl FromStr for ImageFile {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "archive.zip" {
			return Ok(Self::Archive);
		}

		if let Some((mut name, ext)) = s.split_once('.') {
			let extension = ImageFileExtension::from_str(ext)?;

			let is_static = if let Some(n) = name.strip_suffix("_static") {
				name = n;
				true
			} else {
				false
			};

			Ok(Self::Image {
				name: ImageFileName::from_str(name)?,
				extension,
				is_static,
			})
		} else {
			anyhow::bail!("missing file extension")
		}
	}
}

impl FromStr for BadgeFile {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"1x" => Ok(Self::One),
			"2x" => Ok(Self::Two),
			"3x" => Ok(Self::Three),
			_ => anyhow::bail!("invalid badge file"),
		}
	}
}

impl Display for ImageFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ImageFile::Image {
				name,
				extension,
				is_static: false,
			} => write!(f, "{}.{}", name, extension),
			ImageFile::Image {
				name,
				extension,
				is_static: true,
			} => write!(f, "{}_static.{}", name, extension),
			ImageFile::Archive => write!(f, "archive.zip"),
		}
	}
}

impl Display for BadgeFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BadgeFile::One => write!(f, "1x"),
			BadgeFile::Two => write!(f, "2x"),
			BadgeFile::Three => write!(f, "3x"),
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

impl serde::Serialize for BadgeFile {
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
		Ok(s.parse().map_err(|e| serde::de::Error::custom(e))?)
	}
}

impl<'de> serde::Deserialize<'de> for BadgeFile {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(s.parse().map_err(|e| serde::de::Error::custom(e))?)
	}
}
