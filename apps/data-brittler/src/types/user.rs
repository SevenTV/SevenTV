use mongodb::bson::oid::ObjectId;
use shared::database;
use shared::old_types::UserEditorModelPermission;

use super::ImageFile;

#[derive(Debug, serde::Deserialize)]
pub struct User {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	// pub username: String,
	// pub display_name: String,
	#[serde(default, deserialize_with = "super::empty_string_is_none")]
	pub email: Option<String>,
	#[serde(default)]
	pub avatar: Option<UserAvatar>,
	pub avatar_id: Option<String>,
	// pub biography: String,
	pub editors: Vec<UserEditor>,
	pub role_ids: Vec<ObjectId>,
	pub connections: Vec<UserConnection>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum UserAvatar {
	#[allow(dead_code)]
	Pending {
		pending_id: ObjectId,
	},
	Processed {
		// id: ObjectId,
		input_file: ImageFile,
		image_files: Vec<ImageFile>,
	},
	// for some reason one user has an empty object here
	None {},
}

#[derive(Debug, serde::Deserialize)]
pub struct UserEditor {
	#[serde(default)]
	pub id: Option<ObjectId>,
	#[serde(default)]
	pub permissions: UserEditorModelPermission,
	// #[serde(default)]
	// pub visible: bool,
	#[serde(default)]
	pub added_at: super::DateTime,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserConnection {
	// some users are completely missing the "id" field
	// the connections without any id should be ignored
	pub id: Option<String>,
	#[serde(flatten)]
	pub platform: ConnectionPlatform,
	#[serde(default)]
	pub linked_at: super::DateTime,
	pub emote_set_id: Option<ObjectId>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "platform", content = "data")]
pub enum ConnectionPlatform {
	Twitch {
		#[serde(default, deserialize_with = "super::empty_string_is_none")]
		id: Option<String>,
		#[serde(default)]
		login: String,
		display_name: String,
		#[serde(default, deserialize_with = "super::empty_string_is_none")]
		profile_image_url: Option<String>,
	},
	Discord {
		#[serde(default, deserialize_with = "super::empty_string_is_none")]
		id: Option<String>,
		#[serde(default)]
		username: String,
		avatar: String,
	},
	Youtube {
		#[serde(default, deserialize_with = "super::empty_string_is_none")]
		id: Option<String>,
		#[serde(default)]
		title: String,
		#[serde(default, deserialize_with = "super::empty_string_is_none")]
		profile_image_url: Option<String>,
	},
	Kick {
		#[serde(default, deserialize_with = "super::empty_string_is_none")]
		id: Option<String>,
		#[serde(default)]
		username: String,
		display_name: String,
	},
}

impl From<ConnectionPlatform> for database::user::connection::Platform {
	fn from(value: ConnectionPlatform) -> Self {
		match value {
			ConnectionPlatform::Twitch { .. } => Self::Twitch,
			ConnectionPlatform::Discord { .. } => Self::Discord,
			ConnectionPlatform::Youtube { .. } => Self::Google,
			ConnectionPlatform::Kick { .. } => Self::Kick,
		}
	}
}
