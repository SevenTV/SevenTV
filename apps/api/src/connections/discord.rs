use serde::Deserialize;

use super::{ConnectionError, PlatformUserData};

#[derive(Debug, Deserialize)]
pub struct DiscordUserData {
	pub id: String,
	pub username: String,
	pub global_name: Option<String>,
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
			username: value.username.clone(),
			display_name: value.global_name.unwrap_or(value.username),
		}
	}
}

pub async fn get_user_data(access_token: &str) -> Result<DiscordUserData, ConnectionError> {
	let res = reqwest::Client::new()
		.get("https://discord.com/api/v10/users/@me")
		.bearer_auth(access_token)
		.send()
		.await
		.map_err(|err| {
			tracing::error!(error = %err, "request failed");
			ConnectionError::RequestError
		})?;

	let status = res.status();
	let text = res.text().await.map_err(|err| {
		tracing::error!(error = %err, "failed to read response");
		ConnectionError::RequestError
	})?;

	if status.is_success() {
		serde_json::from_str(&text).map_err(|err| {
			tracing::error!(error = %err, text, "failed to parse response");
			ConnectionError::RequestError
		})
	} else {
		tracing::error!(%status, text, "invalid response");
		Err(ConnectionError::RequestError)
	}
}
