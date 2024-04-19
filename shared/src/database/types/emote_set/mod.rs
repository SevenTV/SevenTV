use bitmask_enum::bitmask;

use crate::database::{Collection, Id};
use crate::types::old::{
	ActiveEmoteFlagModel, ActiveEmoteModel, EmotePartialModel, EmoteSetFlagModel, EmoteSetModel, EmoteSetPartialModel,
	UserPartialModel,
};

mod emote;

pub use emote::*;

use super::UserId;

pub type EmoteSetId = Id<EmoteSet>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteSet {
	#[serde(rename = "_id", skip_serializing_if = "Id::is_nil")]
	pub id: EmoteSetId,
	pub owner_id: Option<UserId>,
	pub name: String,
	pub kind: EmoteSetKind,
	pub tags: Vec<String>,
	pub capacity: u32,
	pub flags: EmoteSetFlags,
}

#[bitmask(u8)]
pub enum EmoteSetFlags {
	Immutable = 1 << 0,
	Privileged = 1 << 1,
	Private = 1 << 2,
	Published = 1 << 3,
}

impl Default for EmoteSetFlags {
	fn default() -> Self {
		EmoteSetFlags::none()
	}
}

impl serde::Serialize for EmoteSetFlags {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteSetFlags {
	fn deserialize<D>(deserializer: D) -> Result<EmoteSetFlags, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = u8::deserialize(deserializer)?;
		Ok(EmoteSetFlags::from(bits))
	}
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

				if self.flags.contains(EmoteSetFlags::Immutable) {
					flags |= EmoteSetFlagModel::Immutable;
				}

				if self.flags.contains(EmoteSetFlags::Privileged) {
					flags |= EmoteSetFlagModel::Privileged;
				}

				flags
			},
			tags: self.tags,
			immutable: self.flags.contains(EmoteSetFlags::Immutable),
			privileged: self.flags.contains(EmoteSetFlags::Privileged),
			emote_count: emotes.len() as i32,
			capacity: self.capacity as i32,
			emotes,
			origins: Vec::new(),
			owner,
		}
	}

	pub fn into_old_model_partial(self, owner: Option<UserPartialModel>) -> EmoteSetPartialModel {
		EmoteSetPartialModel {
			id: self.id,
			name: self.name,
			capacity: self.capacity as i32,
			flags: {
				let mut flags = EmoteSetFlagModel::none();

				if self.kind == EmoteSetKind::Personal {
					flags |= EmoteSetFlagModel::Personal;
				}

				if self.flags.contains(EmoteSetFlags::Immutable) {
					flags |= EmoteSetFlagModel::Immutable;
				}

				if self.flags.contains(EmoteSetFlags::Privileged) {
					flags |= EmoteSetFlagModel::Privileged;
				}

				flags
			},
			tags: self.tags,
			owner,
		}
	}
}

impl Collection for EmoteSet {
	const COLLECTION_NAME: &'static str = "emote_sets";
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum EmoteSetKind {
	#[default]
	Normal,
	Personal,
}
