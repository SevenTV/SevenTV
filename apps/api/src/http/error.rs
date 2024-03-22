use hyper::StatusCode;
use scuffle_utils::{database::deadpool_postgres::PoolError, http::RouteError};

use crate::connections::ConnectionError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("connection: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("database: {0}")]
    Database(#[from] PoolError),
}

impl From<scuffle_utils::database::tokio_postgres::Error> for ApiError {
	fn from(value: scuffle_utils::database::tokio_postgres::Error) -> Self {
		Self::Database(value.into())
	}
}

impl From<ApiError> for StatusCode {
    fn from(value: ApiError) -> Self {
        match value {
            ApiError::ConnectionError(ConnectionError::ReqwestError(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ConnectionError(_) => StatusCode::BAD_REQUEST,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ApiError> for RouteError<ApiError> {
    fn from(value: ApiError) -> Self {
        let msg = format!("{:#}", value);
        RouteError::from((StatusCode::from(value), msg))
    }
}
