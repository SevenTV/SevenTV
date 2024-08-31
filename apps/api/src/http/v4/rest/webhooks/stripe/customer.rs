use std::str::FromStr;
use std::sync::Arc;

use shared::database::product::CustomerId;
use shared::database::queries::{filter, update};
use shared::database::user::{User, UserId};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn created(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	customer: stripe::Customer,
) -> TransactionResult<(), ApiError> {
	let Some(user_id) = customer.metadata.as_ref().and_then(|m| m.get("USER_ID")) else {
		// no user id on metadata
		return Ok(());
	};

	let user_id = UserId::from_str(user_id).map_err(|e| {
		tracing::error!(error = %e, "invalid user id");
		TransactionError::custom(ApiError::BAD_REQUEST)
	})?;

	tx.update_one(
		filter::filter! {
			User {
				id: user_id,
			}
		},
		update::update! {
			#[query(set)]
			User {
				stripe_customer_id: Some(CustomerId::from(customer.id)),
			}
		},
		None,
	)
	.await?;

	Ok(())
}
