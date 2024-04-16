use std::fmt::Display;
use std::str::FromStr;

use bson::oid::ObjectId;

use crate::database::Collection;
use crate::types::old::{UserConnectionPartialModel, UserConnectionPlatformModel};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserConnection {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub user_id: ObjectId,
	pub main_connection: bool,
	pub platform: Platform,
	pub platform_id: String,
	pub platform_username: String,
	pub platform_display_name: String,
	pub platform_avatar_url: Option<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub allow_login: bool,
}

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Platform {
	#[default]
	Twitch,
	Discord,
	Google,
	Kick,
}

impl From<Platform> for UserConnectionPlatformModel {
	fn from(value: Platform) -> Self {
		match value {
			Platform::Twitch => Self::Twitch,
			Platform::Discord => Self::Discord,
			Platform::Google => Self::Youtube,
			Platform::Kick => Self::Kick,
		}
	}
}

impl FromStr for Platform {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"twitch" => Ok(Self::Twitch),
			"discord" => Ok(Self::Discord),
			"google" => Ok(Self::Google),
			"kick" => Ok(Self::Kick),
			_ => Err(()),
		}
	}
}

impl Display for Platform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Twitch => write!(f, "twitch"),
			Self::Discord => write!(f, "discord"),
			Self::Google => write!(f, "google"),
			Self::Kick => write!(f, "kick"),
		}
	}
}

impl Collection for UserConnection {
	const NAME: &'static str = "user_connections";
}

impl From<UserConnection> for UserConnectionPartialModel {
	fn from(value: UserConnection) -> Self {
		Self {
			id: value.platform_id,
			platform: value.platform.into(),
			username: value.platform_username,
			display_name: value.platform_display_name,
			linked_at: value.id.timestamp().timestamp_millis(),
			emote_capacity: 600,
			emote_set_id: None,
		}
	}
}
