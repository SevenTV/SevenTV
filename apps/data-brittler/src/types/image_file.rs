use std::str::FromStr;

use anyhow::Context;
use shared::database::image_set::Image;
use shared::database::user::profile_picture::UserProfilePictureId;
use shared::database::Id;

#[derive(Debug, Clone, serde::Deserialize)]
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

impl TryFrom<ImageFile> for Image {
	type Error = anyhow::Error;

	fn try_from(value: ImageFile) -> Result<Self, Self::Error> {
		let mut key = value.key.split('/');

		let ty = key.next().ok_or(anyhow::anyhow!("missing type"))?;

		let scale = match value.name.as_str() {
			"1x_static" | "1x" => 1,
			"2x_static" | "2x" => 2,
			"3x_static" | "3x" => 3,
			"4x_static" | "4x" => 4,
			_ => 1,
		};

		let new_path = if ty == "user" {
			let id = Id::<()>::from_str(key.next().ok_or(anyhow::anyhow!("missing id"))?)
				.with_context(|| format!("invalid id: {}", value.key))?;
			let avatar_id = UserProfilePictureId::from_str(
				key.next()
					.with_context(|| format!("missing avatar id: {}", value.key))?
					.strip_prefix("av_")
					.with_context(|| format!("invalid avatar id: {}", value.key))?,
			)
			.with_context(|| format!("invalid avatar id: {}", value.key))?;
			let file = key.next().with_context(|| format!("missing file: {}", value.key))?;

			format!("/user/{}/profile-picture/{}/{}", id, avatar_id, file)
		} else {
			let next_segment = key.next().with_context(|| format!("missing segment: {}", value.key))?;
			// some paths start with "emote/emote"
			let id = if next_segment == "emote" {
				Id::<()>::from_str(key.next().with_context(|| format!("missing id: {}", value.key))?)
			} else {
				Id::<()>::from_str(next_segment)
			}
			.with_context(|| format!("invalid id: {}", value.key))?;
			let file = key.next().with_context(|| format!("missing file: {}", value.key))?;

			format!("/{}/{}/{}", ty, id, file)
		};

		Ok(Self {
			path: new_path,
			scale,
			mime: value.content_type,
			size: value.size as i64,
			width: value.width as i32,
			height: value.height as i32,
			frame_count: value.frame_count as i32,
		})
	}
}
