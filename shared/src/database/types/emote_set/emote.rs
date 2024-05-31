use bitmask_enum::bitmask;

use super::EmoteSetId;
use crate::database::{Collection, EmoteId, Id, UserId};

pub type EmoteSetEmoteId = Id<EmoteSetEmote>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetEmote {
	#[serde(rename = "_id")]
	pub id: EmoteSetEmoteId,
	pub emote_set_id: EmoteSetId,
	pub emote_id: EmoteId,
	pub added_by_id: Option<UserId>,
	pub name: String,
	pub flags: EmoteSetEmoteFlag,
}

#[bitmask(i32)]
pub enum EmoteSetEmoteFlag {
	ZeroWidth = 1 << 0,
	OverrideConflicts = 1 << 1,
}

impl Default for EmoteSetEmoteFlag {
	fn default() -> Self {
		EmoteSetEmoteFlag::none()
	}
}

impl serde::Serialize for EmoteSetEmoteFlag {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteSetEmoteFlag {
	fn deserialize<D>(deserializer: D) -> Result<EmoteSetEmoteFlag, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = i32::deserialize(deserializer)?;
		Ok(EmoteSetEmoteFlag::from(bits))
	}
}

impl Collection for EmoteSetEmote {
	const COLLECTION_NAME: &'static str = "emote_set_emotes";
}
