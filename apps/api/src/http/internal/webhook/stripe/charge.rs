use std::sync::Arc;

use axum::response::IntoResponse;

use crate::global::Global;
use crate::http::error::ApiError;

pub async fn refund(global: Arc<Global>, subscription: stripe::Charge) -> Result<impl IntoResponse, ApiError> {
	Ok(())
}
