use std::sync::Arc;

use axum::http::StatusCode;

use crate::{global::Global, http::error::ApiError};

use super::types;

pub async fn expired(global: &Arc<Global>, subscription: types::Subscription) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn cancelled(global: &Arc<Global>, subscription: types::Subscription) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn suspended(global: &Arc<Global>, subscription: types::Subscription) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn payment_failed(global: &Arc<Global>, subscription: types::Subscription) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}
