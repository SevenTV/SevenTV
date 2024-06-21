use std::sync::Arc;

use bitmask_enum::bitmask;

use super::{ImageSet, UserId};
use crate::database::{Collection, Id};

pub type EmoteId = Id<Emote>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Emote {
	#[serde(rename = "_id")]
	pub id: EmoteId,
	pub owner_id: Option<UserId>,
	pub default_name: String,
	pub tags: Vec<String>,
	pub animated: bool,
	pub image_set: ImageSet,
	pub flags: EmoteFlags,
	pub attribution: Vec<EmoteAttribution>,
	pub merged_into: Option<EmoteId>,
	pub merged_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Collection for Emote {
	const COLLECTION_NAME: &'static str = "emotes";
}

#[bitmask(u8)]
pub enum EmoteFlags {
	PublicListed = 1 << 0,
	Private = 1 << 1,
	Nsfw = 1 << 2,
	DefaultZeroWidth = 1 << 3,
	ApprovedPersonal = 1 << 4,
	DeniedPersonal = 1 << 5,
}

impl Default for EmoteFlags {
	fn default() -> Self {
		Self::none()
	}
}

impl serde::Serialize for EmoteFlags {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteFlags {
	fn deserialize<D>(deserializer: D) -> Result<EmoteFlags, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = u8::deserialize(deserializer)?;
		Ok(EmoteFlags::from(bits))
	}
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteAttribution {
	pub user_id: UserId,
	#[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
	pub added_at: chrono::DateTime<chrono::Utc>,
}
