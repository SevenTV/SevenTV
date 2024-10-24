use std::fmt::Display;
use std::str::FromStr;

use crate::database::badge::BadgeId;
use crate::database::emote::EmoteId;
use crate::database::paint::{PaintId, PaintLayerId};
use crate::database::user::profile_picture::UserProfilePictureId;
use crate::database::user::UserId;

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
		avatar_id: UserProfilePictureId,
		file: ImageFile,
	},
	Paint {
		paint_id: PaintId,
		layer_id: PaintLayerId,
		file: ImageFile,
	},
}

impl serde::Serialize for CacheKey {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> serde::Deserialize<'de> for CacheKey {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		s.parse().map_err(serde::de::Error::custom)
	}
}

impl FromStr for CacheKey {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut splits = s.split('/');
		let prefix = splits.next().ok_or("invalid cache key")?;

		let key = match prefix {
			"badge" => Self::Badge {
				badge_id: splits
					.next()
					.ok_or("invalid cache key")?
					.parse()
					.map_err(|_| "invalid cache key")?,
				file: splits.next().ok_or("invalid cache key")?.parse()?,
			},
			"emote" => Self::Emote {
				emote_id: splits
					.next()
					.ok_or("invalid cache key")?
					.parse()
					.map_err(|_| "invalid cache key")?,
				file: splits.next().ok_or("invalid cache key")?.parse()?,
			},
			"user" => {
				let user_id = splits
					.next()
					.ok_or("invalid cache key")?
					.parse()
					.map_err(|_| "invalid cache key")?;
				match splits.next().ok_or("invalid cache key")? {
					"profile-picture" => Self::UserProfilePicture {
						user_id,
						avatar_id: splits
							.next()
							.ok_or("invalid cache key")?
							.parse()
							.map_err(|_| "invalid cache key")?,
						file: splits.next().ok_or("invalid cache key")?.parse()?,
					},
					_ => return Err("invalid cache key"),
				}
			}
			"paint" => {
				let paint_id = splits
					.next()
					.ok_or("invalid cache key")?
					.parse()
					.map_err(|_| "invalid cache key")?;
				match splits.next().ok_or("invalid cache key")? {
					"layer" => Self::Paint {
						paint_id,
						layer_id: splits
							.next()
							.ok_or("invalid cache key")?
							.parse()
							.map_err(|_| "invalid cache key")?,
						file: splits.next().ok_or("invalid cache key")?.parse()?,
					},
					_ => return Err("invalid cache key"),
				}
			}
			_ => return Err("invalid cache key"),
		};

		Ok(key)
	}
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
