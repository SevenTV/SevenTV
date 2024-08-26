use std::sync::Arc;

use crate::{global::Global, http::error::ApiError, transactions::{TransactionError, TransactionResult, TransactionSession}};

use super::types;

pub async fn created(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	dispute: types::Dispute,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn updated(global: &Arc<Global>, tx: TransactionSession<'_, ApiError>, dispute: types::Dispute) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn resolved(global: &Arc<Global>, tx: TransactionSession<'_, ApiError>, dispute: types::Dispute) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}
