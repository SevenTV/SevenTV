use std::sync::Arc;

use serde::Deserialize;

use super::{ConnectionError, PlatformUserData};
use crate::global::Global;

#[derive(Debug, Deserialize)]
struct TwitchResponse {
	pub data: Vec<TwitchUserData>,
}

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
	let res = global
		.http_client
		.get("https://api.twitch.tv/helix/users")
		.header("Client-Id", global.config.api.connections.twitch.client_id.clone())
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
		let res = serde_json::from_str::<TwitchResponse>(&text).map_err(|err| {
			tracing::error!(error = %err, text, "failed to parse response");
			ConnectionError::RequestError
		})?;

		Ok(res.data.into_iter().next().ok_or(ConnectionError::NoUserData)?)
	} else {
		tracing::error!(%status, text, "invalid response");
		Err(ConnectionError::RequestError)
	}
}
