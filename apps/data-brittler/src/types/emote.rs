use mongodb::bson::oid::ObjectId;
use shared::old_types::EmoteFlagsModel;

use super::image_file::ImageFile;

#[derive(Debug, serde::Deserialize)]
pub struct Emote {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub name: String,
	pub flags: EmoteFlagsModel,
	pub owner_id: ObjectId,
	pub tags: Vec<String>,
	pub versions: Vec<EmoteVersion>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EmoteVersion {
	pub id: ObjectId,
	#[serde(default, deserialize_with = "super::empty_string_is_none")]
	pub name: Option<String>,
	#[serde(default, deserialize_with = "super::empty_string_is_none")]
	pub description: Option<String>,
	pub animated: bool,
	pub state: EmoteVersionState,
	pub input_file: ImageFile,
	pub image_files: Vec<ImageFile>,
	pub created_at: super::DateTime,
}

#[derive(Debug, serde::Deserialize)]
pub struct EmoteVersionState {
	pub listed: bool,
	#[serde(default, deserialize_with = "super::null_to_default")]
	pub allow_personal: bool,
	pub lifecycle: EmoteLifecycle,
	pub replace_id: Option<ObjectId>,
}

#[derive(Debug, PartialEq, Eq, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum EmoteLifecycle {
	Deleted = -1,
	Pending = 0,
	Processing = 1,
	Disabled = 2,
	Live = 3,
	Failed = -2,
}
