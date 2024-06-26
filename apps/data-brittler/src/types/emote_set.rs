use mongodb::bson::oid::ObjectId;
use shared::old_types::{ActiveEmoteFlagModel, EmoteSetFlagModel};

#[derive(Debug, serde::Deserialize)]
pub struct EmoteSet {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub owner_id: ObjectId,
	pub name: String,
	pub capacity: i32,
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
	#[serde(default, deserialize_with = "super::empty_string_is_none")]
	pub name: Option<String>,
	#[serde(default)]
	pub flags: ActiveEmoteFlagModel,
	pub actor_id: Option<ObjectId>,
	pub timestamp: Option<super::DateTime>,
}
