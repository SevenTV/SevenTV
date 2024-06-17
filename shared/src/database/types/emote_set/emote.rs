use bitmask_enum::bitmask;

use super::EmoteSetId;
use crate::database::emote::EmoteId;
use crate::database::user::UserId;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetEmote {
	pub id: EmoteId,
	pub alias: String,
	pub added_at: chrono::DateTime<chrono::Utc>,
	pub flags: EmoteSetEmoteFlag,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub added_by_id: Option<UserId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub origin_set_id: Option<EmoteSetId>,
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
