use std::str::FromStr;

use postgres_types::{FromSql, ToSql};

use crate::database::Table;
use crate::global::Global;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserConnection {
	pub id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub platform: UserConnectionPlatform,
	pub platform_id: String,
	pub platform_username: String,
	pub platform_display_name: String,
	pub platform_avatar: String,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: UserConnectionSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserConnectionSettings {}

impl Table for UserConnection {
	const TABLE_NAME: &'static str = "user_connections";
}
