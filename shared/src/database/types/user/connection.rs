use std::fmt::Display;
use std::str::FromStr;

use super::UserId;
use crate::database::{Collection, Id};

pub type UserConnectionId = Id<UserConnection>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserConnection {
	#[serde(rename = "_id")]
	pub id: UserConnectionId,
	pub user_id: UserId,
	pub main_connection: bool,
	pub platform: Platform,
	pub platform_id: String,
	pub platform_username: String,
	pub platform_display_name: String,
	pub platform_avatar_url: Option<String>,
	pub allow_login: bool,
}

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum Platform {
	#[default]
	Twitch = 0,
	Discord = 1,
	Google = 2,
	Kick = 3,
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
	const COLLECTION_NAME: &'static str = "user_connections";
}
