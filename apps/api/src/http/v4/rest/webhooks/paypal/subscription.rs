use std::sync::Arc;

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

use super::types;

pub async fn expired(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	subscription: types::Subscription,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn cancelled(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	subscription: types::Subscription,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn suspended(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	subscription: types::Subscription,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn payment_failed(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	subscription: types::Subscription,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}
