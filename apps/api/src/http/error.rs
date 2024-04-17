use crate::connections::ConnectionError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("connection: {0}")]
	ConnectionError(#[from] ConnectionError),
	#[error("json: {0}")]
	Json(#[from] serde_json::Error),
	#[error("http: {0}")]
	Http(#[from] hyper::http::Error),
	#[error("mongo: {0}")]
	Mongo(#[from] mongodb::error::Error),
}
