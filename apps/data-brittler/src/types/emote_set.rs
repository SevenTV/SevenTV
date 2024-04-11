use shared::object_id::ObjectId;
pub use shared::types::old::{ActiveEmoteFlagModel, EmoteSetFlagModel};

#[derive(Debug, serde::Deserialize)]
pub struct EmoteSet {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub owner_id: ObjectId,
	pub name: String,
	#[serde(deserialize_with = "super::unsigned_int")]
	pub capacity: u32,
	#[serde(default)]
	pub immutable: bool,
	#[serde(default)]
	pub privileged: bool,
	#[serde(default)]
	pub flags: EmoteSetFlagModel,
	#[serde(default, deserialize_with = "super::null_to_default")]
	pub tags: Vec<String>,
	pub emotes: Vec<Option<EmoteSetEmote>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EmoteSetEmote {
	pub id: Option<ObjectId>,
	pub name: String,
	#[serde(default)]
	pub flags: ActiveEmoteFlagModel,
	pub actor_id: Option<ObjectId>,
	pub timestamp: Option<super::DateTime>,
}
