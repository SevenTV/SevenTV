use std::sync::Arc;

use axum::http::StatusCode;

use crate::{global::Global, http::error::ApiError};

use super::types;

pub async fn created(global: &Arc<Global>, dispute: types::Dispute) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn updated(global: &Arc<Global>, dispute: types::Dispute) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn resolved(global: &Arc<Global>, dispute: types::Dispute) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}
