use postgres_types::{FromSql, ToSql};
use ulid::Ulid;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserConnection {
	pub id: Ulid,
	pub user_id: Ulid,
	pub platform: UserConnectionPlatform,
	pub platform_id: String,
	pub platform_username: String,
	pub platform_avatar: String,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: UserConnectionSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
pub enum UserConnectionPlatform {
	#[default]
	#[postgres(name = "TWITCH")]
	Twitch,
	#[postgres(name = "DISCORD")]
	Discord,
	#[postgres(name = "YOUTUBE")]
	Youtube,
	#[postgres(name = "KICK")]
	Kick,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserConnectionSettings {}
