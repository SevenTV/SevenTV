use std::sync::Arc;

use shared::database::product::CustomerId;
use shared::database::queries::{filter, update};
use shared::database::user::User;

use crate::global::Global;
use crate::http::egvault::metadata::{CustomerMetadata, StripeMetadata};
use crate::http::error::{ApiError, ApiErrorCode};
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn created(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	customer: stripe::Customer,
) -> TransactionResult<(), ApiError> {
	let Some(metadata) = customer
		.metadata
		.as_ref()
		.filter(|m| !m.is_empty())
		.map(CustomerMetadata::from_stripe)
		.transpose()
		.map_err(|err| {
			tracing::error!(error = %err, "failed to deserialize metadata");
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to deserialize metadata",
			))
		})?
	else {
		// no metadata
		return Ok(());
	};

	tx.update_one(
		filter::filter! {
			User {
				id: metadata.user_id,
			}
		},
		update::update! {
			#[query(set)]
			User {
				stripe_customer_id: Some(CustomerId::from(customer.id)),
				updated_at: chrono::Utc::now(),
				search_updated_at: &None,
			}
		},
		None,
	)
	.await?;

	Ok(())
}
