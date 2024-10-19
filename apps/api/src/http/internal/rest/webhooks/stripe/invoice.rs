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
use shared::database::stripe_errors::{StripeError, StripeErrorId, StripeErrorKind};
use stripe::{FinalizeInvoiceParams, Object};

use crate::global::Global;
use crate::http::egvault::metadata::{CustomerMetadata, InvoiceMetadata, StripeMetadata, SubscriptionMetadata};
use crate::http::error::{ApiError, ApiErrorCode};
use crate::stripe_client::SafeStripeClient;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

fn invoice_items(items: Option<&stripe::List<stripe::InvoiceLineItem>>) -> Result<Vec<ProductId>, ApiError> {
	items
		.ok_or_else(|| ApiError::bad_request(ApiErrorCode::StripeError, "invoice line items are missing"))?
		.data
		.iter()
		.map(|line| {
			Ok(line
				.price
				.as_ref()
				.ok_or_else(|| ApiError::bad_request(ApiErrorCode::StripeError, "invoice line item price is missing"))?
				.id()
				.into())
		})
		.collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StripeRequest {
	CreatedRetrieveSubscription,
	CreatedRetrieveCustomer,
	CreatedFinalizeInvoice,
	PaidRetrieveSubscription,
}

impl std::fmt::Display for StripeRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CreatedRetrieveSubscription => write!(f, "created_retrieve_subscription"),
			Self::CreatedRetrieveCustomer => write!(f, "created_retrieve_customer"),
			Self::CreatedFinalizeInvoice => write!(f, "created_finalize_invoice"),
			Self::PaidRetrieveSubscription => write!(f, "paid_retrieve_subscription"),
		}
	}
}

/// Creates the invoice object and finalize it.
pub async fn created(
	_global: &Arc<Global>,
	stripe_client: SafeStripeClient<super::StripeRequest>,
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
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to deserialize metadata",
			))
		})?;

	let customer_id = invoice
		.customer
		.ok_or_else(|| {
			TransactionError::Custom(ApiError::bad_request(
				ApiErrorCode::StripeError,
				"invoice customer is missing",
			))
		})?
		.id();

	// Invoices are only created for subscriptions
	let user_id = match (invoice.subscription, metadata) {
		(Some(subscription), _) => {
			let subscription = stripe::Subscription::retrieve(
				stripe_client
					.client(super::StripeRequest::Invoice(StripeRequest::CreatedRetrieveSubscription))
					.await
					.deref(),
				&subscription.id(),
				&[],
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to retrieve subscription",
				))
			})?;

			let metadata = SubscriptionMetadata::from_stripe(&subscription.metadata).map_err(|e| {
				tracing::error!(error = %e, "failed to deserialize metadata");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to deserialize metadata",
				))
			})?;

			metadata.customer_id.unwrap_or(metadata.user_id)
		}
		(None, Some(InvoiceMetadata::Gift { customer_id, .. })) => customer_id,
		_ => {
			let customer = stripe::Customer::retrieve(
				stripe_client
					.client(super::StripeRequest::Invoice(StripeRequest::CreatedRetrieveCustomer))
					.await
					.deref(),
				&customer_id,
				&[],
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve customer");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to retrieve customer",
				))
			})?;

			let metadata = CustomerMetadata::from_stripe(&customer.metadata.unwrap_or_default()).map_err(|e| {
				tracing::error!(error = %e, "failed to deserialize metadata");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to deserialize metadata",
				))
			})?;

			metadata.user_id
		}
	};

	let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::Custom)?;

	let status = invoice
		.status
		.ok_or_else(|| {
			TransactionError::Custom(ApiError::bad_request(ApiErrorCode::StripeError, "invoice status is missing"))
		})?
		.into();

	let created_at = invoice
		.created
		.and_then(|t| chrono::DateTime::from_timestamp(t, 0))
		.ok_or_else(|| {
			TransactionError::Custom(ApiError::bad_request(
				ApiErrorCode::StripeError,
				"invoice created_at is missing",
			))
		})?;

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
			stripe_client
				.client(super::StripeRequest::Invoice(StripeRequest::CreatedFinalizeInvoice))
				.await
				.deref(),
			&invoice.id,
			FinalizeInvoiceParams {
				auto_advance: Some(true),
			},
		)
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to finalize invoice");
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to finalize invoice",
			))
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

	let status: InvoiceStatus = invoice
		.status
		.ok_or_else(|| {
			TransactionError::Custom(ApiError::bad_request(ApiErrorCode::StripeError, "invoice status is missing"))
		})?
		.into();

	let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::Custom)?;

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
				search_updated_at: &None,
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
	stripe_client: SafeStripeClient<super::StripeRequest>,
	mut tx: TransactionSession<'_, ApiError>,
	event_id: stripe::EventId,
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
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to deserialize metadata",
			))
		})?;

	match (invoice.subscription, metadata) {
		(Some(subscription), _) => {
			let items = invoice_items(invoice.lines.as_ref())
				.map_err(TransactionError::Custom)?
				.into_iter()
				.collect::<Vec<_>>();

			if items.len() != 1 {
				tx.insert_one(
					StripeError {
						id: StripeErrorId::new(),
						event_id,
						error_kind: StripeErrorKind::SubscriptionInvoiceInvalidItems,
					},
					None,
				)
				.await?;

				return Ok(None);
			}

			let stripe_product_id = items.into_iter().next().unwrap();

			let Some(product) = tx
				.find_one(
					filter::filter! {
						SubscriptionProduct {
							#[query(flatten)]
							variants: SubscriptionProductVariant {
								id: &stripe_product_id,
							}
						}
					},
					None,
				)
				.await?
			else {
				tx.insert_one(
					StripeError {
						id: StripeErrorId::new(),
						event_id,
						error_kind: StripeErrorKind::SubscriptionInvoiceNoProduct,
					},
					None,
				)
				.await?;

				return Ok(None);
			};

			// This invoice is for one of our subscription products.

			// Retrieve subscription
			let stripe_sub = stripe::Subscription::retrieve(
				stripe_client
					.client(super::StripeRequest::Invoice(StripeRequest::PaidRetrieveSubscription))
					.await
					.deref(),
				&subscription.id(),
				&[],
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to retrieve subscription",
				))
			})?;

			let user_id = SubscriptionMetadata::from_stripe(&stripe_sub.metadata)
				.map_err(|e| {
					tracing::error!(error = %e, "failed to deserialize metadata");
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to deserialize metadata",
					))
				})?
				.user_id;

			let sub_id = SubscriptionId {
				user_id,
				product_id: product.id,
			};

			let start = chrono::DateTime::from_timestamp(stripe_sub.current_period_start, 0).ok_or_else(|| {
				TransactionError::Custom(ApiError::bad_request(
					ApiErrorCode::StripeError,
					"subscription current period start is missing",
				))
			})?;
			// when the subscription is in trial, the current period end is the trial end
			let end = chrono::DateTime::from_timestamp(stripe_sub.current_period_end, 0).ok_or_else(|| {
				TransactionError::Custom(ApiError::bad_request(
					ApiErrorCode::StripeError,
					"subscription current period end is missing",
				))
			})?;

			let provider_id = ProviderSubscriptionId::from(stripe_sub.id);

			tx.insert_one(
				SubscriptionPeriod {
					id: SubscriptionPeriodId::new(),
					subscription_id: sub_id,
					provider_id: Some(provider_id),
					start,
					end,
					product_id: stripe_product_id,
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
				product_id,
				subscription_product_id: Some(subscription_product_id),
			}),
		) => {
			// gift code session
			// the gift sub payment was successful, now we add one subscription period for
			// the recipient

			let subscription_product = global
				.subscription_product_by_id_loader
				.load(subscription_product_id)
				.await
				.map_err(|_| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to load subscription product",
					))
				})?
				.ok_or_else(|| {
					tracing::warn!(
						"could not find subscription product for gift: {} product id: {subscription_product_id}",
						invoice.id
					);
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to load subscription product",
					))
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
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to load subscription product variant",
					))
				})?;

			let created = invoice.created.ok_or_else(|| {
				TransactionError::Custom(ApiError::bad_request(
					ApiErrorCode::StripeError,
					"invoice created_at is missing",
				))
			})?;

			let start = chrono::DateTime::from_timestamp(created, 0).ok_or_else(|| {
				TransactionError::Custom(ApiError::bad_request(
					ApiErrorCode::StripeError,
					"invoice created_at is missing",
				))
			})?;

			let end = start
				.checked_add_months(period_duration) // It's fine to use this function here since UTC doens't have daylight saving time transitions
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"invoice created_at is missing",
					))
				})?;

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
					product_id,
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
				search_updated_at: &None,
			}
		},
		None,
	)
	.await?;

	Ok(())
}
