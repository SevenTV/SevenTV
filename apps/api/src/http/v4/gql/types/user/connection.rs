#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct UserConnection {
	pub platform: Platform,
	pub platform_id: String,
	pub platform_username: String,
	pub platform_display_name: String,
	pub platform_avatar_url: Option<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub linked_at: chrono::DateTime<chrono::Utc>,
	pub allow_login: bool,
}

impl From<shared::database::user::connection::UserConnection> for UserConnection {
	fn from(value: shared::database::user::connection::UserConnection) -> Self {
		Self {
			platform: value.platform.into(),
			platform_id: value.platform_id,
			platform_username: value.platform_username,
			platform_display_name: value.platform_display_name,
			platform_avatar_url: value.platform_avatar_url,
			updated_at: value.updated_at,
			linked_at: value.linked_at,
			allow_login: value.allow_login,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, async_graphql::Enum)]
pub enum Platform {
	Twitch,
	Discord,
	Google,
	Kick,
}

impl From<Platform> for shared::database::user::connection::Platform {
	fn from(value: Platform) -> Self {
		match value {
			Platform::Twitch => Self::Twitch,
			Platform::Discord => Self::Discord,
			Platform::Google => Self::Google,
			Platform::Kick => Self::Kick,
		}
	}
}

impl From<shared::database::user::connection::Platform> for Platform {
	fn from(value: shared::database::user::connection::Platform) -> Self {
		match value {
			shared::database::user::connection::Platform::Twitch => Self::Twitch,
			shared::database::user::connection::Platform::Discord => Self::Discord,
			shared::database::user::connection::Platform::Google => Self::Google,
			shared::database::user::connection::Platform::Kick => Self::Kick,
		}
	}
}
