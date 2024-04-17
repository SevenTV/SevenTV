use bitmask_enum::bitmask;
use bson::oid::ObjectId;

use crate::database::Collection;
use crate::types::old::{ActiveEmoteFlagModel, ActiveEmoteModel, EmotePartialModel};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetEmote {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub emote_set_id: ObjectId,
	pub emote_id: ObjectId,
	pub added_by_id: Option<ObjectId>,
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

impl EmoteSetEmote {
	pub fn into_old_model(self, data: Option<EmotePartialModel>) -> ActiveEmoteModel {
		ActiveEmoteModel {
			id: self.emote_id,
			actor_id: self.added_by_id,
			name: self.name,
			timestamp: self.id.timestamp().timestamp_millis(),
			origin_id: None,
			flags: {
				let mut flags = ActiveEmoteFlagModel::none();

				if self.flags.contains(EmoteSetEmoteFlag::ZeroWidth) {
					flags |= ActiveEmoteFlagModel::ZeroWidth;
				}

				if self.flags.contains(EmoteSetEmoteFlag::OverrideConflicts) {
					flags |= ActiveEmoteFlagModel::OverrideBetterTTV
						| ActiveEmoteFlagModel::OverrideTwitchGlobal
						| ActiveEmoteFlagModel::OverrideTwitchSubscriber;
				}

				flags
			},
			data,
		}
	}
}

impl Collection for EmoteSetEmote {
	const COLLECTION_NAME: &'static str = "emote_set_emotes";
}
