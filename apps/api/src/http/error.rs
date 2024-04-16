use scuffle_utils::database::deadpool_postgres::PoolError;

use crate::connections::ConnectionError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("connection: {0}")]
	ConnectionError(#[from] ConnectionError),
	#[error("database: {0}")]
	Database(#[from] PoolError),
	#[error("json: {0}")]
	Json(#[from] serde_json::Error),
	#[error("http: {0}")]
	Http(#[from] hyper::http::Error),
}

impl From<scuffle_utils::database::tokio_postgres::Error> for ApiError {
	fn from(value: scuffle_utils::database::tokio_postgres::Error) -> Self {
		Self::Database(value.into())
	}
}
