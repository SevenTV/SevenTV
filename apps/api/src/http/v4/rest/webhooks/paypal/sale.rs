use std::sync::Arc;

use axum::http::StatusCode;

use crate::{global::Global, http::error::ApiError};

use super::types;

pub async fn completed(global: &Arc<Global>, sale: types::Sale) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn refunded(global: &Arc<Global>, sale: types::Sale) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn reversed(global: &Arc<Global>, sale: types::Sale) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}
