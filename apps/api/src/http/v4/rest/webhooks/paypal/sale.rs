use std::sync::Arc;

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

use super::types;

pub async fn completed(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	sale: types::Sale,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn refunded(global: &Arc<Global>, tx: TransactionSession<'_, ApiError>, sale: types::Sale) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn reversed(global: &Arc<Global>, tx: TransactionSession<'_, ApiError>, sale: types::Sale) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}
