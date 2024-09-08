use std::ops::Deref;
use std::sync::Arc;

use shared::database::product::invoice::{Invoice, InvoiceStatus};
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId,
};
use shared::database::product::{
	InvoiceId, ProductId, SubscriptionProduct, SubscriptionProductKind, SubscriptionProductVariant,
};
use shared::database::queries::{filter, update};
use stripe::{FinalizeInvoiceParams, Object};

use crate::global::Global;
use crate::http::egvault::metadata::{CustomerMetadata, InvoiceMetadata, StripeMetadata, SubscriptionMetadata};
use crate::http::error::ApiError;
use crate::stripe_client::SafeStripeClient;
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
	_global: &Arc<Global>,
	stripe_client: SafeStripeClient,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	let metadata = invoice
		.metadata
		.as_ref()
		.map(InvoiceMetadata::from_stripe)
		.transpose()
		.map_err(|err| {
			tracing::error!(error = %err, "failed to deserialize metadata");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

	let customer_id = invoice.customer.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.id();

	// Invoices are only created for subscriptions
	let user_id = match (invoice.subscription, metadata) {
		(Some(subscription), _) => {
			let subscription =
				stripe::Subscription::retrieve(stripe_client.client(0).await.deref(), &subscription.id(), &[])
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to retrieve subscription");
						TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
					})?;

			let metadata = SubscriptionMetadata::from_stripe(&subscription.metadata).map_err(|e| {
				tracing::error!(error = %e, "failed to deserialize metadata");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

			metadata.customer_id.unwrap_or(metadata.user_id)
		}
		(None, Some(InvoiceMetadata::Gift { customer_id, .. })) => customer_id,
		_ => {
			let customer = stripe::Customer::retrieve(stripe_client.client(1).await.deref(), &customer_id, &[])
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to retrieve customer");
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;

			let metadata = CustomerMetadata::from_stripe(&customer.metadata.unwrap_or_default()).map_err(|e| {
				tracing::error!(error = %e, "failed to deserialize metadata");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

			metadata.user_id
		}
	};

	let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::custom)?;

	let status = invoice.status.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.into();

	let created_at = invoice
		.created
		.and_then(|t| chrono::DateTime::from_timestamp(t, 0))
		.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

	let db_invoice = Invoice {
		id: invoice.id.clone().into(),
		items,
		customer_id: customer_id.into(),
		user_id,
		paypal_payment_id: None,
		status,
		failed: false,
		refunded: false,
		disputed: None,
		created_at,
		updated_at: chrono::Utc::now(),
		search_updated_at: None,
	};

	tx.insert_one(db_invoice, None).await?;

	if invoice.status == Some(stripe::InvoiceStatus::Draft) {
		stripe::Invoice::finalize(
			stripe_client.client(2).await.deref(),
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
	tx: &mut TransactionSession<'_, ApiError>,
	invoice: &stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	let id: InvoiceId = invoice.id.clone().into();

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
	stripe_client: SafeStripeClient,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	updated(global, &mut tx, &invoice).await?;

	let metadata = invoice
		.metadata
		.as_ref()
		.map(InvoiceMetadata::from_stripe)
		.transpose()
		.map_err(|err| {
			tracing::error!(error = %err, "failed to deserialize metadata");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;

	match (invoice.subscription, metadata) {
		(Some(subscription), _) => {
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
			let stripe_sub = stripe::Subscription::retrieve(stripe_client.client(0).await.deref(), &subscription.id(), &[])
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to retrieve subscription");
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;

			let user_id = SubscriptionMetadata::from_stripe(&stripe_sub.metadata)
				.map_err(|e| {
					tracing::error!(error = %e, "failed to deserialize metadata");
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?
				.user_id;

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
					subscription_id: sub_id,
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

			return Ok(Some(sub_id));
		}
		(
			None,
			Some(InvoiceMetadata::Gift {
				customer_id,
				user_id,
				subscription_product_id: Some(subscription_product_id),
			}),
		) => {
			// gift code session
			// the gift sub payment was successful, now we add one subscription period for
			// the recipient
			let items = invoice_items(invoice.lines.as_ref())
				.map_err(TransactionError::custom)?
				.into_iter()
				.collect::<Vec<_>>();

			if items.len() != 1 {
				// only accept invoices for one of our products
				return Ok(None);
			}

			let product_id = items.into_iter().next().unwrap();

			let subscription_product = global
				.subscription_product_by_id_loader
				.load(subscription_product_id)
				.await
				.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
				.ok_or_else(|| {
					tracing::warn!(
						"could not find subscription product for gift: {} product id: {subscription_product_id}",
						invoice.id
					);
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;

			let period_duration = subscription_product
				.variants
				.iter()
				.find(|v| v.id == product_id)
				.map(|v| match v.kind {
					SubscriptionProductKind::Monthly => chrono::Months::new(1),
					SubscriptionProductKind::Yearly => chrono::Months::new(12),
				})
				.ok_or_else(|| {
					tracing::warn!(
						"could not find variant for gift: {} product id: {subscription_product_id}",
						invoice.id
					);
					TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
				})?;

			let created = invoice.created.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

			let start =
				chrono::DateTime::from_timestamp(created, 0).ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

			let end = start
				.checked_add_months(period_duration) // It's fine to use this function here since UTC doens't have daylight saving time transitions
				.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

			let sub_id = SubscriptionId {
				user_id,
				product_id: subscription_product_id,
			};

			tx.insert_one(
				SubscriptionPeriod {
					id: SubscriptionPeriodId::new(),
					subscription_id: sub_id,
					provider_id: None,
					start,
					end,
					is_trial: false,
					created_by: SubscriptionPeriodCreatedBy::Gift {
						gifter: customer_id,
						invoice: invoice.id.into(),
					},
					updated_at: chrono::Utc::now(),
					search_updated_at: None,
				},
				None,
			)
			.await?;

			return Ok(Some(sub_id));
		}
		_ => {}
	}

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
