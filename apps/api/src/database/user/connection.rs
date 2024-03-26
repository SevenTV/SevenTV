use std::str::FromStr;

use postgres_types::{FromSql, ToSql};

use crate::database::Table;
use crate::global::Global;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserConnection {
	pub id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub platform: UserConnectionPlatform,
	pub platform_access_token: String,
	pub platform_access_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
	pub platform_refresh_token: Option<String>,
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

impl ToString for UserConnectionPlatform {
	fn to_string(&self) -> String {
		match self {
			Self::Twitch => "twitch",
			Self::Discord => "discord",
			Self::Google => "google",
			Self::Kick => "kick",
		}
		.to_string()
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserConnectionSettings {}

impl Table for UserConnection {
	const TABLE_NAME: &'static str = "user_connections";
}
