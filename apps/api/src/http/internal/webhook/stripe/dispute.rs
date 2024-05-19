use std::sync::Arc;

use axum::response::IntoResponse;

use crate::global::Global;
use crate::http::error::ApiError;

pub async fn created(global: Arc<Global>, subscription: stripe::Dispute) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn updated(global: Arc<Global>, subscription: stripe::Dispute) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn closed(global: Arc<Global>, subscription: stripe::Dispute) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}
