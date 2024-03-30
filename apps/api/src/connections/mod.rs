use std::sync::Arc;

use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::database::UserConnectionPlatform;
use crate::global::Global;

mod discord;
mod google;
mod twitch;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
	#[error("unsupported platform")]
	UnsupportedPlatform,
	#[error("invalid response code: {0:?}")]
	InvalidResponse(StatusCode),
	#[error("no user data")]
	NoUserData,
	#[error("login not allowed")]
	LoginNotAllowed,
	#[error("reqwest: {0}")]
	ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug, Serialize)]
struct TokenRequest {
	grant_type: String,
	code: String,
	client_id: String,
	client_secret: String,
	redirect_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
	pub token_type: String,
	pub access_token: String,
	pub expires_in: i64,
	pub refresh_token: Option<String>,
	pub scope: Option<TokenResponseScope>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TokenResponseScope {
	Twitch(Vec<String>),
	Other(String),
}

/// Twitch docs: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
/// Discord docs: https://discord.com/developers/docs/topics/oauth2#authorization-code-grant
/// Google docs: https://developers.google.com/identity/protocols/oauth2/web-server#exchange-authorization-code
pub async fn exchange_code(
	global: &Arc<Global>,
	platform: UserConnectionPlatform,
	code: &str,
	redirect_uri: String,
) -> Result<TokenResponse, ConnectionError> {
	let (url, config) = match platform {
		UserConnectionPlatform::Twitch => ("https://id.twitch.tv/oauth2/token", &global.config().api.connections.twitch),
		UserConnectionPlatform::Discord => (
			"https://discord.com/api/v10/oauth2/token",
			&global.config().api.connections.discord,
		),
		UserConnectionPlatform::Google => ("https://oauth2.googleapis.com/token", &global.config().api.connections.google),
		_ => return Err(ConnectionError::UnsupportedPlatform),
	};
	let req = TokenRequest {
		grant_type: "authorization_code".to_string(),
		code: code.to_string(),
		client_id: config.client_id.to_string(),
		client_secret: config.client_secret.to_string(),
		redirect_uri,
	};
	let res = global.http_client().post(url).form(&req).send().await?;
	let status = res.status();
	if status.is_success() {
		Ok(res.json().await?)
	} else {
		Err(ConnectionError::InvalidResponse(status))
	}
}

pub struct PlatformUserData {
	pub id: String,
	pub username: String,
	pub display_name: String,
	pub avatar: Option<String>,
}

pub async fn get_user_data(
	global: &Arc<Global>,
	platform: UserConnectionPlatform,
	access_token: &str,
) -> Result<PlatformUserData, ConnectionError> {
	match platform {
		UserConnectionPlatform::Discord => discord::get_user_data(access_token).await.map(Into::into),
		UserConnectionPlatform::Google => google::get_user_data(access_token).await.map(Into::into),
		UserConnectionPlatform::Twitch => twitch::get_user_data(global, access_token).await.map(Into::into),
		_ => Err(ConnectionError::UnsupportedPlatform),
	}
}
