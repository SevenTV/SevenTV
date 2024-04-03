use ulid::Ulid;

use super::{EmoteSetModel, UserModel, UserPartialModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UserConnectionModel {
    pub id: String,
    pub platform: UserConnectionPlatformModel,
    pub username: String,
    pub display_name: String,
    pub linked_at: i64,
    pub emote_capacity: i32,
    pub emote_set_id: Option<Ulid>,
    pub emote_set: Option<EmoteSetModel>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub presences: Vec<UserPartialModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UserConnectionPartialModel {
    pub id: String,
    pub platform: UserConnectionPlatformModel,
    pub username: String,
    pub display_name: String,
    pub linked_at: i64,
    pub emote_capacity: i32,
    pub emote_set_id: Option<Ulid>,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserConnectionPlatformModel {
    #[default]
    Twitch,
    Youtube,
    Discord,
    Kick,
}
