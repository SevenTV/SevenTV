use std::sync::Arc;

use serde::Deserialize;

use super::{ConnectionError, PlatformUserData};
use crate::global::Global;

#[derive(Debug, Deserialize)]
pub struct TwitchUserData {
	pub id: String,
	pub login: String,
	pub display_name: String,
	pub profile_image_url: Option<String>,
}

impl From<TwitchUserData> for PlatformUserData {
	fn from(value: TwitchUserData) -> Self {
		Self {
			id: value.id,
			username: value.login,
			display_name: value.display_name,
			avatar: value.profile_image_url,
		}
	}
}

pub async fn get_user_data(global: &Arc<Global>, access_token: &str) -> Result<TwitchUserData, ConnectionError> {
	Ok(reqwest::Client::new()
		.get("https://api.twitch.tv/helix/users")
		.header("Client-Id", global.config().api.connections.twitch.client_id.clone())
		.bearer_auth(access_token)
		.send()
		.await?
		.json()
		.await?)
}
