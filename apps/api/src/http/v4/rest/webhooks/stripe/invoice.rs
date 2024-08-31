use std::{collections::HashMap, str::FromStr, sync::Arc};

use shared::database::{
	product::{
		invoice::{Invoice, InvoiceStatus},
		subscription::{
			ProviderSubscriptionId, Subscription, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId,
		},
		InvoiceId, ProductId,
	},
	queries::{filter, update},
	user::UserId,
};
use stripe::{FinalizeInvoiceParams, Object};

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

fn invoice_items(items: Option<&stripe::List<stripe::InvoiceLineItem>>) -> Result<Vec<ProductId>, ApiError> {
	// TODO: paginate?
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
	if let Some(subscription) = invoice.subscription {
		// We have to fetch the subscription here to determine the id of the user this invoice is for.
		let subscription = stripe::Subscription::retrieve(&global.stripe_client, &subscription.id(), &[])
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

		let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::custom)?;

		let subscription_products = global
			.subscription_product_by_id_loader
			.load_many(items.iter().cloned())
			.await
			.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		if subscription_products.is_empty() {
			// This invoice is not for one of our subscription products.
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

		Ok(())
	} else {
		// TODO: do something here?
		Ok(())
	}
}

/// Updates the invoice object.
///
/// Called for `invoice.updated`, `invoice.finalized`, `invoice.payment_succeeded`
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

/// If invoice is for a subscription, adds a new subscription period to that subscription.
/// If the subscription is still in trial, makes the period a trial period.
/// Updates the invoice object.
///
/// Called for `invoice.paid`
pub async fn paid(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	if let Some(subscription) = &invoice.subscription {
		let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::custom)?;

		let products = global
			.subscription_product_by_id_loader
			.load_many(items.iter().cloned())
			.await
			.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

		if !products.is_empty() {
			// This invoice is for one of our subscription products.

			// Retrieve subscription
			let subscription = stripe::Subscription::retrieve(&global.stripe_client, &subscription.id(), &[])
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to retrieve subscription");
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;

			let user_id = subscription
				.metadata
				.get("USER_ID")
				.and_then(|i| UserId::from_str(i).ok())
				.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

			let start = chrono::DateTime::from_timestamp(subscription.current_period_start, 0)
				.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;
			// when the subscription is in trial, the current period end is the trial end
			let end = chrono::DateTime::from_timestamp(subscription.current_period_end, 0)
				.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

			let subscription_id = ProviderSubscriptionId::from(subscription.id);

			tx.insert_one(
				SubscriptionPeriod {
					id: SubscriptionPeriodId::new(),
					subscription_id: Some(subscription_id.clone()),
					user_id,
					start,
					end,
					is_trial: subscription.trial_end.is_some(),
					created_by: SubscriptionPeriodCreatedBy::Invoice {
						invoice_id: invoice.id.clone().into(),
					},
					product_ids: items,
					updated_at: chrono::Utc::now(),
					search_updated_at: None,
				},
				None,
			)
			.await?;

			tx.update_one(
				filter::filter! {
					Subscription {
						#[query(rename = "_id", serde)]
						id: subscription_id,
					}
				},
				update::update! {
					#[query(set)]
					Subscription {
						cancel_at_period_end: false,
						ended_at: Option::<chrono::DateTime<chrono::Utc>>::None,
						updated_at: chrono::Utc::now(),
					}
				},
				None,
			)
			.await?;
		}
	}

	updated(global, tx, invoice, HashMap::new()).await?;

	Ok(())
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
/// Should prompt the user to collect new payment information and update the subscription's default payment method afterwards.
pub async fn payment_failed(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	// TODO: Show the user an error message.
	// TODO: Collect new payment information and update the subscriptions default payment method.

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
