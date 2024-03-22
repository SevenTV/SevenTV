use serde::Deserialize;

use super::{ConnectionError, PlatformUserData};

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

pub async fn get_user_data(access_token: &str) -> Result<YoutubeUserData, ConnectionError> {
	Ok(reqwest::Client::new()
		.get("https://youtube.googleapis.com/youtube/v3/channels?part=snippet&mine=true")
		.bearer_auth(access_token)
		.send()
		.await?
		.json()
		.await?)
}
