use std::ops::Deref;
use std::sync::Arc;

use shared::database::product::invoice::{Invoice, InvoiceDisputeStatus};
use shared::database::product::InvoiceId;
use shared::database::queries::{filter, update};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::stripe_client::SafeStripeClient;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

/// Marks associated invoice as refunded.
/// Creates a ticket to let staff know that the refund was made and decide what
/// should happen next.
pub async fn refunded(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	charge: stripe::Charge,
) -> TransactionResult<(), ApiError> {
	let Some(invoice_id) = charge.invoice.map(|i| InvoiceId::from(i.id())) else {
		return Ok(());
	};

	tx.update_one(
		filter::filter! {
			Invoice {
				#[query(rename = "_id")]
				id: invoice_id,
			}
		},
		update::update! {
			#[query(set)]
			Invoice {
				#[query(serde)]
				refunded: true,
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}

/// Marks the associated invoice as disputed.
///
/// Called for `charge.dispute.created`, `charge.dispute.updated`,
/// `charge.dispute.closed`
pub async fn dispute_updated(
	_global: &Arc<Global>,
	stripe_client: SafeStripeClient<super::StripeRequest>,
	mut tx: TransactionSession<'_, ApiError>,
	dispute: stripe::Dispute,
) -> TransactionResult<(), ApiError> {
	let charge = stripe::Charge::retrieve(
		stripe_client.client(super::StripeRequest::Charge).await.deref(),
		&dispute.charge.id(),
		&[],
	)
	.await
	.map_err(|e| {
		tracing::error!(error = %e, "failed to retrieve charge");
		TransactionError::Custom(ApiError::internal_server_error(
			ApiErrorCode::StripeError,
			"failed to retrieve charge",
		))
	})?;

	let Some(invoice_id) = charge.invoice.map(|i| InvoiceId::from(i.id())) else {
		return Ok(());
	};

	let disputed: Option<InvoiceDisputeStatus> = Some(dispute.status.into());

	tx.update_one(
		filter::filter! {
			Invoice {
				#[query(rename = "_id")]
				id: invoice_id,
			}
		},
		update::update! {
			#[query(set)]
			Invoice {
				#[query(serde)]
				disputed,
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}
