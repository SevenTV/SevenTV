use std::sync::Arc;

use shared::database::product::invoice::{Invoice, InvoiceDisputeStatus};
use shared::database::queries::{filter, update};

use super::types;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionResult, TransactionSession};

/// Called for `CUSTOMER.DISPUTE.CREATED`, `CUSTOMER.DISPUTE.UPDATED`,
/// `CUSTOMER.DISPUTE.RESOLVED`
pub async fn updated(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	dispute: types::Dispute,
) -> TransactionResult<(), ApiError> {
	let payment_ids: Vec<_> = dispute
		.disputed_transactions
		.into_iter()
		.map(|t| t.seller_transaction_id)
		.collect();

	let disputed: InvoiceDisputeStatus = dispute.status.into();

	tx.update_one(
		filter::filter! {
			Invoice {
				#[query(selector = "in")]
				paypal_payment_id: payment_ids,
			}
		},
		update::update! {
			#[query(set)]
			Invoice {
				#[query(serde)]
				disputed: Some(disputed),
				updated_at: chrono::Utc::now(),
				search_updated_at: &None,
			}
		},
		None,
	)
	.await?;

	Ok(())
}
