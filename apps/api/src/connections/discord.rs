use serde::Deserialize;

use super::{ConnectionError, PlatformUserData};

#[derive(Debug, Deserialize)]
pub struct DiscordUserData {
	pub id: String,
	pub username: String,
	pub global_name: String,
	pub avatar: Option<String>,
}

impl From<DiscordUserData> for PlatformUserData {
	fn from(value: DiscordUserData) -> Self {
		// https://discord.com/developers/docs/reference#image-formatting
		let avatar = value.avatar.map(|a| {
			let ext = if a.starts_with("a_") { "gif" } else { "png" };
			format!("https://cdn.discordapp.com/avatars/{}/{}.{ext}", value.id, a)
		});
		Self {
			avatar,
			id: value.id,
			username: value.username,
			display_name: value.global_name,
		}
	}
}

pub async fn get_user_data(access_token: &str) -> Result<DiscordUserData, ConnectionError> {
	Ok(reqwest::Client::new()
		.get("https://discord.com/api/v10/users/@me")
		.bearer_auth(access_token)
		.send()
		.await?
		.json()
		.await?)
}
