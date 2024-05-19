use std::sync::Arc;

use axum::response::IntoResponse;

use crate::global::Global;
use crate::http::error::ApiError;

pub async fn created(global: Arc<Global>, invoice: stripe::Invoice) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn updated(global: Arc<Global>, invoice: stripe::Invoice) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn deleted(global: Arc<Global>, invoice: stripe::Invoice) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn paid(global: Arc<Global>, invoice: stripe::Invoice) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn payment_failed(global: Arc<Global>, invoice: stripe::Invoice) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn voided(global: Arc<Global>, invoice: stripe::Invoice) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}

pub async fn marked_uncollectible(global: Arc<Global>, invoice: stripe::Invoice) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}
