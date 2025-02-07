use std::sync::Arc;

use serde::Deserialize;

use super::{ConnectionError, PlatformUserData};
use crate::global::Global;

#[derive(Debug, Deserialize)]
struct KickResponse {
	pub data: Vec<KickUserData>,
}

#[derive(Debug, Deserialize)]
pub struct KickUserData {
	// pub email: String,
	pub name: String,
	pub profile_picture: Option<String>,
	pub user_id: i32,
}

impl From<KickUserData> for PlatformUserData {
	fn from(value: KickUserData) -> Self {
		Self {
			id: value.user_id.to_string(),
			username: value.name.clone(),
			display_name: value.name,
			avatar: value.profile_picture,
		}
	}
}

pub async fn get_user_data(global: &Arc<Global>, access_token: &str) -> Result<KickUserData, ConnectionError> {
	let res = global
		.http_client
		.get("https://api.kick.com/users")
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
		let res = serde_json::from_str::<KickResponse>(&text).map_err(|err| {
			tracing::error!(error = %err, text, "failed to parse response");
			ConnectionError::RequestError
		})?;

		Ok(res.data.into_iter().next().ok_or(ConnectionError::NoUserData)?)
	} else {
		tracing::error!(%status, text, "invalid response");
		Err(ConnectionError::RequestError)
	}
}
