use bitmask_enum::bitmask;

use crate::database::{Collection, Id};

mod emote;

pub use emote::*;

use super::UserId;

pub type EmoteSetId = Id<EmoteSet>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteSet {
	#[serde(rename = "_id")]
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
	/// immutable, cannot be modified
	Immutable = 1 << 0,
	/// can only be modified by the owner
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

impl Collection for EmoteSet {
	const COLLECTION_NAME: &'static str = "emote_sets";
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum EmoteSetKind {
	#[default]
	Normal = 0,
	Personal = 1,
}
