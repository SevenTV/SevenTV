use shared::object_id::ObjectId;
use shared::types::old::EmoteFlagsModel;

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
}