use std::sync::Arc;

use serde::{Deserialize, Serialize};
use shared::database::user::connection::Platform;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

mod discord;
mod google;
mod twitch;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
	#[error("unsupported platform")]
	UnsupportedPlatform,
	#[error("request failed")]
	RequestError,
	#[error("no user data")]
	NoUserData,
}

impl From<ConnectionError> for ApiError {
	fn from(value: ConnectionError) -> Self {
		match value {
			ConnectionError::UnsupportedPlatform => ApiError::bad_request(ApiErrorCode::BadRequest, "unsupported platform"),
			ConnectionError::RequestError => ApiError::internal_server_error(ApiErrorCode::LoadError, "request failed"),
			ConnectionError::NoUserData => {
				ApiError::bad_request(ApiErrorCode::LoadError, "3rd party platform did not return user data")
			}
		}
	}
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
	pub access_token: String,
}

/// Twitch docs: https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
/// Discord docs: https://discord.com/developers/docs/topics/oauth2#authorization-code-grant
/// Google docs: https://developers.google.com/identity/protocols/oauth2/web-server#exchange-authorization-code
/// Kick docs: Kick does not have any documentation.
#[tracing::instrument(skip(global, code, redirect_uri), fields(endpoint))]
pub async fn exchange_code(
	global: &Arc<Global>,
	platform: Platform,
	code: &str,
	redirect_uri: String,
) -> Result<TokenResponse, ConnectionError> {
	let (endpoint, config) = match platform {
		Platform::Twitch => ("https://id.twitch.tv/oauth2/token", &global.config.connections.twitch),
		Platform::Discord => ("https://discord.com/api/v10/oauth2/token", &global.config.connections.discord),
		Platform::Google => ("https://oauth2.googleapis.com/token", &global.config.connections.google),
		_ => return Err(ConnectionError::UnsupportedPlatform),
	};

	tracing::Span::current().record("endpoint", endpoint);

	let res = global
		.http_client
		.post(endpoint)
		.form(&TokenRequest {
			grant_type: "authorization_code".to_string(),
			code: code.to_string(),
			client_id: config.client_id.to_string(),
			client_secret: config.client_secret.to_string(),
			redirect_uri,
		})
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
		Ok(serde_json::from_str(&text).map_err(|err| {
			tracing::error!(error = %err, text, "failed to parse response");
			ConnectionError::RequestError
		})?)
	} else {
		tracing::error!(%status, text, "invalid response");
		Err(ConnectionError::RequestError)
	}
}

pub struct PlatformUserData {
	pub id: String,
	pub username: String,
	pub display_name: String,
	pub avatar: Option<String>,
}

#[tracing::instrument(skip(global, access_token))]
pub async fn get_user_data(
	global: &Arc<Global>,
	platform: Platform,
	access_token: &str,
) -> Result<PlatformUserData, ConnectionError> {
	match platform {
		Platform::Discord => discord::get_user_data(global, access_token).await.map(Into::into),
		Platform::Google => google::get_user_data(global, access_token).await.map(Into::into),
		Platform::Twitch => twitch::get_user_data(global, access_token).await.map(Into::into),
		_ => Err(ConnectionError::UnsupportedPlatform),
	}
}
