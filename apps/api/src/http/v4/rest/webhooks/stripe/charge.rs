use std::sync::Arc;

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

pub async fn refunded(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	charge: stripe::Charge,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn dispute_created(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	charge: stripe::Charge,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn dispute_updated(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	charge: stripe::Charge,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn dispute_closed(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	charge: stripe::Charge,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}
