use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use shared::database::product::invoice::{Invoice, InvoiceStatus};
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId,
};
use shared::database::product::{InvoiceId, ProductId, SubscriptionProduct, SubscriptionProductVariant};
use shared::database::queries::{filter, update};
use shared::database::user::UserId;
use stripe::{FinalizeInvoiceParams, Object};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

fn invoice_items(items: Option<&stripe::List<stripe::InvoiceLineItem>>) -> Result<Vec<ProductId>, ApiError> {
	items
		.ok_or(ApiError::BAD_REQUEST)?
		.data
		.iter()
		.map(|line| Ok(line.price.as_ref().ok_or(ApiError::BAD_REQUEST)?.id().into()))
		.collect()
}

/// Creates the invoice object and finalize it.
pub async fn created(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	// Invoices are only created for subscriptions
	if let Some(subscription) = invoice.subscription {
		// We have to fetch the subscription here to determine the id of the user this
		// invoice is for.
		let subscription = stripe::Subscription::retrieve(&global.stripe_client, &subscription.id(), &[])
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

		let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::custom)?;

		let subscription_products = tx
			.find(
				filter::filter! {
					SubscriptionProduct {
						#[query(flatten)]
						variants: SubscriptionProductVariant {
							#[query(selector = "in")]
							id: &items,
						},
					}
				},
				None,
			)
			.await?;

		if subscription_products.len() != 1 {
			// This invoice is not for one of our subscription products.
			// only accept invoices for one of our products
			return Ok(());
		}

		let customer_id = invoice
			.customer
			.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?
			.id()
			.into();

		let user_id = subscription
			.metadata
			.get("USER_ID")
			.and_then(|i| UserId::from_str(i).ok())
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let status = invoice.status.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.into();

		let created_at = invoice
			.created
			.and_then(|t| chrono::DateTime::from_timestamp(t, 0))
			.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

		let db_invoice = Invoice {
			id: invoice.id.clone().into(),
			items,
			customer_id,
			user_id,
			paypal_payment_id: None,
			status,
			failed: false,
			refunded: false,
			disputed: None,
			created_at,
			updated_at: created_at,
			search_updated_at: None,
		};

		tx.insert_one(db_invoice, None).await?;

		stripe::Invoice::finalize(
			&global.stripe_client,
			&invoice.id,
			FinalizeInvoiceParams {
				auto_advance: Some(true),
			},
		)
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to finalize invoice");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;
	}

	Ok(())
}

/// Updates the invoice object.
///
/// Called for `invoice.updated`, `invoice.finalized`,
/// `invoice.payment_succeeded`
pub async fn updated(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
	_prev_attributes: HashMap<String, serde_json::Value>,
) -> TransactionResult<(), ApiError> {
	let id: InvoiceId = invoice.id.into();

	let status: InvoiceStatus = invoice.status.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.into();

	let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::custom)?;

	tx.update_one(
		filter::filter! {
			Invoice {
				#[query(rename = "_id")]
				id,
			}
		},
		update::update! {
			#[query(set)]
			Invoice {
				#[query(serde)]
				status: status,
				#[query(serde)]
				items: items,
				failed: false,
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}

/// If invoice is for a subscription, adds a new subscription period to that
/// subscription. If the subscription is still in trial, makes the period a
/// trial period. Updates the invoice object.
///
/// Called for `invoice.paid`
pub async fn paid(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	if let Some(subscription) = &invoice.subscription {
		let items = invoice_items(invoice.lines.as_ref())
			.map_err(TransactionError::custom)?
			.into_iter()
			.collect::<Vec<_>>();

		let products = tx
			.find(
				filter::filter! {
					SubscriptionProduct {
						#[query(flatten)]
						variants: SubscriptionProductVariant {
							#[query(selector = "in", serde)]
							id: items,
						}
					}
				},
				None,
			)
			.await?;

		if products.len() != 1 {
			// only accept invoices for one of our products
			return Ok(None);
		}

		let product = products.into_iter().next().unwrap();

		// This invoice is for one of our subscription products.

		// Retrieve subscription
		let stripe_sub = stripe::Subscription::retrieve(&global.stripe_client, &subscription.id(), &[])
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

		let user_id = stripe_sub
			.metadata
			.get("USER_ID")
			.and_then(|i| UserId::from_str(i).ok())
			.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		let sub_id = SubscriptionId {
			user_id,
			product_id: product.id,
		};

		let start = chrono::DateTime::from_timestamp(stripe_sub.current_period_start, 0)
			.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;
		// when the subscription is in trial, the current period end is the trial end
		let end = chrono::DateTime::from_timestamp(stripe_sub.current_period_end, 0)
			.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

		let provider_id = ProviderSubscriptionId::from(stripe_sub.id);

		tx.insert_one(
			SubscriptionPeriod {
				id: SubscriptionPeriodId::new(),
				subscription_id: sub_id.clone(),
				provider_id: Some(provider_id),
				start,
				end,
				is_trial: stripe_sub.trial_end.is_some(),
				created_by: SubscriptionPeriodCreatedBy::Invoice {
					invoice_id: invoice.id.clone().into(),
					cancel_at_period_end: stripe_sub.cancel_at_period_end,
				},
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
			None,
		)
		.await?;

		updated(global, tx, invoice, HashMap::new()).await?;

		return Ok(Some(sub_id));
	}

	updated(global, tx, invoice, HashMap::new()).await?;

	Ok(None)
}

/// Only sent for draft invoices.
/// Deletes the invoice object.
pub async fn deleted(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	let id: InvoiceId = invoice.id.into();

	tx.find_one_and_delete(
		filter::filter! {
			Invoice {
				#[query(rename = "_id")]
				id,
			}
		},
		None,
	)
	.await?;

	Ok(())
}

/// Marks the associated invoice as failed.
/// Shows the user an error message.
/// Should prompt the user to collect new payment information and update the
/// subscription's default payment method afterwards.
pub async fn payment_failed(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	// TODO: Show the user an error message.
	// TODO: Collect new payment information and update the subscriptions default
	// payment method.

	let id: InvoiceId = invoice.id.into();

	tx.update_one(
		filter::filter! {
			Invoice {
				#[query(rename = "_id")]
				id,
			}
		},
		update::update! {
			#[query(set)]
			Invoice {
				failed: true,
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}
