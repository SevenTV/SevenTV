use std::sync::Arc;

use serde::Deserialize;

use super::{ConnectionError, PlatformUserData};
use crate::global::Global;

#[derive(Debug, Deserialize)]
struct YoutubeResponse {
	pub items: Vec<YoutubeUserData>,
}

#[derive(Debug, Deserialize)]
pub struct YoutubeUserData {
	pub id: String,
	pub snippet: YoutubeUserSnippet,
}

#[derive(Debug, Deserialize)]
pub struct YoutubeUserSnippet {
	pub title: String,
	#[serde(rename(deserialize = "customUrl"))]
	pub custom_url: Option<String>,
	pub thumbnails: Option<YoutubeUserThumbnails>,
}

#[derive(Debug, Deserialize)]
pub struct YoutubeUserThumbnails {
	pub default: YoutubeUserThumbnail,
	pub medium: YoutubeUserThumbnail,
	pub high: YoutubeUserThumbnail,
}

#[derive(Debug, Deserialize)]
pub struct YoutubeUserThumbnail {
	pub url: String,
	pub width: u64,
	pub height: u64,
}

impl From<YoutubeUserData> for PlatformUserData {
	fn from(value: YoutubeUserData) -> Self {
		Self {
			id: value.id,
			username: value.snippet.custom_url.unwrap_or_else(|| value.snippet.title.clone()),
			display_name: value.snippet.title,
			avatar: value.snippet.thumbnails.map(|t| t.default.url),
		}
	}
}

pub async fn get_user_data(global: &Arc<Global>, access_token: &str) -> Result<YoutubeUserData, ConnectionError> {
	let res = global
		.http_client
		.get("https://youtube.googleapis.com/youtube/v3/channels?part=snippet&mine=true")
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
		let res = serde_json::from_str::<YoutubeResponse>(&text).map_err(|err| {
			tracing::error!(error = %err, text, "failed to parse response");
			ConnectionError::RequestError
		})?;

		Ok(res.items.into_iter().next().ok_or(ConnectionError::NoUserData)?)
	} else {
		tracing::error!(%status, text, "invalid response");
		Err(ConnectionError::RequestError)
	}
}
