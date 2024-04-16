use bitmask_enum::bitmask;
use crate::types::old::{
	ActiveEmoteFlagModel, ActiveEmoteModel, EmotePartialModel, EmoteSetFlagModel, EmoteSetModel, UserPartialModel,
};

use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct EmoteSet {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub name: String,
	pub kind: EmoteSetKind,
	pub tags: Vec<String>,
	pub settings: EmoteSetSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl EmoteSet {
	pub fn into_old_model(
		self,
		emotes: impl IntoIterator<Item = (EmoteSetEmote, Option<EmotePartialModel>)>,
		owner: Option<UserPartialModel>,
	) -> EmoteSetModel {
		let emotes = emotes
			.into_iter()
			.map(|(emote, data)| emote.into_old_model(data))
			.collect::<Vec<_>>();

		EmoteSetModel {
			id: self.id,
			name: self.name,
			flags: {
				let mut flags = EmoteSetFlagModel::none();

				if self.kind == EmoteSetKind::Personal {
					flags |= EmoteSetFlagModel::Personal;
				}

				if self.settings.immutable {
					flags |= EmoteSetFlagModel::Immutable;
				}

				if self.settings.privileged {
					flags |= EmoteSetFlagModel::Privileged;
				}

				flags
			},
			tags: self.tags,
			immutable: self.settings.immutable,
			privileged: self.settings.privileged,
			emote_count: emotes.len() as i32,
			capacity: self.settings.capacity as i32,
			emotes,
			origins: Vec::new(),
			owner,
		}
	}
}

impl Table for EmoteSet {
	const TABLE_NAME: &'static str = "emote_sets";
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum EmoteSetKind {
	#[default]
	Normal,
	Personal,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct EmoteSetSettings {
	pub capacity: u32,
	pub privileged: bool,
	pub immutable: bool,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct EmoteSetEmote {
	pub emote_set_id: ulid::Ulid,
	pub emote_id: ulid::Ulid,
	pub added_by_id: Option<ulid::Ulid>,
	pub name: String,
	pub flags: EmoteSetEmoteFlag,
	pub added_at: chrono::DateTime<chrono::Utc>,
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
			timestamp: self.added_at.timestamp_millis(),
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

impl Table for EmoteSetEmote {
	const TABLE_NAME: &'static str = "emote_set_emotes";
}
