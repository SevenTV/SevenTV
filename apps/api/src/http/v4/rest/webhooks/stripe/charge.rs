use std::sync::Arc;

use axum::http::StatusCode;

use crate::{global::Global, http::error::ApiError};

pub async fn refunded(global: &Arc<Global>, charge: stripe::Charge) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn dispute_created(global: &Arc<Global>, charge: stripe::Charge) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn dispute_updated(global: &Arc<Global>, charge: stripe::Charge) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn dispute_closed(global: &Arc<Global>, charge: stripe::Charge) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}
