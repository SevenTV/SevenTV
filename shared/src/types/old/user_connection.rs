use bson::oid::ObjectId;

use super::{EmoteSetModel, UserModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-connection.model.go#L10
pub struct UserConnectionModel {
	pub id: String,
	pub platform: UserConnectionPlatformModel,
	pub username: String,
	pub display_name: String,
	pub linked_at: i64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<ObjectId>,
	pub emote_set: Option<EmoteSetModel>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<UserModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-connection.model.go#L33
pub struct UserConnectionPartialModel {
	pub id: String,
	pub platform: UserConnectionPlatformModel,
	pub username: String,
	pub display_name: String,
	pub linked_at: i64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<ObjectId>,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-connection.model.go#L49
pub enum UserConnectionPlatformModel {
	#[default]
	Twitch,
	Youtube,
	Discord,
	Kick,
}
