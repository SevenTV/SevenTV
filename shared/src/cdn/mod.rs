pub mod key;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PurgeRequest {
	pub files: Vec<key::CacheKey>,
}
