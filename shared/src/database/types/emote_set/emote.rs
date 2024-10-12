use bitmask_enum::bitmask;

use super::EmoteSetId;
use crate::database::emote::EmoteId;
use crate::database::user::UserId;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetEmote {
	pub id: EmoteId,
	pub alias: String,
	#[serde(with = "crate::database::serde")]
	pub added_at: chrono::DateTime<chrono::Utc>,
	pub flags: EmoteSetEmoteFlags,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub added_by_id: Option<UserId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub origin_set_id: Option<EmoteSetId>,
}

#[bitmask(i32)]
pub enum EmoteSetEmoteFlags {
	ZeroWidth = 1 << 0,
	OverrideConflicts = 1 << 1,
}

impl Default for EmoteSetEmoteFlags {
	fn default() -> Self {
		EmoteSetEmoteFlags::none()
	}
}

impl serde::Serialize for EmoteSetEmoteFlags {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteSetEmoteFlags {
	fn deserialize<D>(deserializer: D) -> Result<EmoteSetEmoteFlags, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = i32::deserialize(deserializer)?;
		Ok(EmoteSetEmoteFlags::from(bits))
	}
}
