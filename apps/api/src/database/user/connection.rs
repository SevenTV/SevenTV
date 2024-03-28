use std::fmt::Display;
use std::str::FromStr;

use postgres_types::{FromSql, ToSql};
use shared::types::UserConnectionPartial;

use crate::database::Table;
use crate::global::Global;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserConnection {
	pub id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub main_connection: bool,
	pub platform: UserConnectionPlatform,
	pub platform_id: String,
	pub platform_username: String,
	pub platform_display_name: String,
	pub platform_avatar_url: Option<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub allow_login: bool,
}

#[derive(Debug, Clone, Copy, Hash, Default, ToSql, FromSql, PartialEq, Eq)]
#[postgres(name = "user_connection_platform")]
pub enum UserConnectionPlatform {
	#[default]
	#[postgres(name = "TWITCH")]
	Twitch,
	#[postgres(name = "DISCORD")]
	Discord,
	#[postgres(name = "GOOGLE")]
	Google,
	#[postgres(name = "KICK")]
	Kick,
}

impl FromStr for UserConnectionPlatform {
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

impl Display for UserConnectionPlatform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Twitch => write!(f, "twitch"),
			Self::Discord => write!(f, "discord"),
			Self::Google => write!(f, "google"),
			Self::Kick => write!(f, "kick"),
		}
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserConnectionSettings {}

impl Table for UserConnection {
	const TABLE_NAME: &'static str = "user_connections";
}

impl From<UserConnection> for UserConnectionPartial {
	fn from(value: UserConnection) -> Self {
		Self {
			id: value.id.into(),
			platform: value.platform.to_string(),
			username: value.platform_username,
			display_name: value.platform_display_name,
			linked_at: value.id.timestamp_ms(),
			emote_capacity: 600,
			emote_set_id: None,
		}
	}
}
