use std::sync::Arc;

use axum::http::StatusCode;

use crate::{global::Global, http::error::ApiError};

pub async fn created(global: &Arc<Global>, subscription: stripe::Subscription) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn deleted(global: &Arc<Global>, subscription: stripe::Subscription) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn updated(global: &Arc<Global>, subscription: stripe::Subscription) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}
