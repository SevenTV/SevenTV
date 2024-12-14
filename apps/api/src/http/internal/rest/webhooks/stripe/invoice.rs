use std::ops::Deref;
use std::sync::Arc;

use chrono::TimeZone;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::invoice::{Invoice, InvoiceStatus};
use shared::database::product::special_event::SpecialEvent;
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId,
};
use shared::database::product::{
	InvoiceId, ProductId, SubscriptionProduct, SubscriptionProductId, SubscriptionProductKind, SubscriptionProductVariant,
};
use shared::database::queries::{filter, update};
use shared::database::stripe_errors::{StripeError, StripeErrorId, StripeErrorKind};
use shared::database::user::UserId;
use stripe::Object;
use tracing::Instrument;

use crate::global::Global;
use crate::http::egvault::metadata::{CustomerMetadata, InvoiceMetadata, StripeMetadata, SubscriptionMetadata};
use crate::http::error::{ApiError, ApiErrorCode};
use crate::paypal_api;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StripeRequest {
	CreatedRetrieveSubscription,
	CreatedRetrieveCustomer,
	PaidRetrieveSubscription,
	CancelSubscription(String),
}

impl std::fmt::Display for StripeRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CreatedRetrieveSubscription => write!(f, "created_retrieve_subscription"),
			Self::CreatedRetrieveCustomer => write!(f, "created_retrieve_customer"),
			Self::PaidRetrieveSubscription => write!(f, "paid_retrieve_subscription"),
			Self::CancelSubscription(id) => write!(f, "cancel_subscription:{}", id),
		}
	}
}

/// Creates the invoice object and finalize it.
#[tracing::instrument(skip_all, name = "stripe::invoice::created")]
pub async fn created(
	_global: &Arc<Global>,
	stripe_client: SafeStripeClient<super::StripeRequest>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	let metadata = invoice
		.metadata
		.as_ref()
		.filter(|m| !m.is_empty())
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
	let user_id = match (invoice.subscription, &metadata) {
		(_, Some(InvoiceMetadata::PaypalLegacy { .. })) => {
			return Ok(());
		}
		(Some(subscription), _) => {
			let subscription = stripe::Subscription::retrieve(
				stripe_client
					.client(super::StripeRequest::Invoice(StripeRequest::CreatedRetrieveSubscription))
					.await
					.deref(),
				&subscription.id(),
				&[],
			)
			.instrument(tracing::info_span!("subscription_retrieve"))
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to retrieve subscription",
				))
			})?;

			// This subscription was not created by us,
			if subscription.metadata.is_empty() {
				tracing::warn!("subscription metadata is missing: {}", subscription.id());
				return Ok(());
			}

			let metadata = SubscriptionMetadata::from_stripe(&subscription.metadata).map_err(|e| {
				tracing::error!(error = %e, "failed to deserialize metadata");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to deserialize subscription metadata",
				))
			})?;

			metadata.customer_id.unwrap_or(metadata.user_id)
		}
		(None, Some(InvoiceMetadata::Gift { customer_id, .. })) => *customer_id,
		_ => {
			let customer = stripe::Customer::retrieve(
				stripe_client
					.client(super::StripeRequest::Invoice(StripeRequest::CreatedRetrieveCustomer))
					.await
					.deref(),
				&customer_id,
				&[],
			)
			.instrument(tracing::info_span!("customer_retrieve"))
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

	Ok(())
}

/// Updates the invoice object.
///
/// Called for `invoice.updated`, `invoice.finalized`,
/// `invoice.payment_succeeded`
#[tracing::instrument(skip_all, name = "stripe::invoice::updated")]
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
#[tracing::instrument(skip_all, name = "stripe::invoice::paid")]
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
		.filter(|m| !m.is_empty())
		.map(InvoiceMetadata::from_stripe)
		.transpose()
		.map_err(|err| {
			tracing::error!(error = %err, "failed to deserialize metadata");
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::StripeError,
				"failed to deserialize metadata",
			))
		})?;

	if let Some(InvoiceMetadata::PaypalLegacy { .. }) = metadata {
		// ignore legacy paypal invoices
		return Ok(None);
	}

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
			.instrument(tracing::info_span!("subscription_retrieve"))
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::Custom(ApiError::internal_server_error(
					ApiErrorCode::StripeError,
					"failed to retrieve subscription",
				))
			})?;

			if stripe_sub.metadata.is_empty() {
				tracing::warn!("subscription metadata is missing: {}", stripe_sub.id);
				return Ok(None);
			}

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
					auto_renew: !stripe_sub.cancel_at_period_end,
					gifted_by: None,
					created_by: SubscriptionPeriodCreatedBy::Invoice {
						invoice_id: invoice.id.clone().into(),
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

			// TODO: Delete after christmas gifter event
			handle_xmas_2024_gift(&mut tx, start, customer_id, subscription_product_id).await?;

			let subscription_id = SubscriptionId {
				user_id,
				product_id: subscription_product_id,
			};

			// Get their current subscription periods
			let current_periods = tx
				.find(
					filter::filter! {
						SubscriptionPeriod {
							#[query(serde)]
							subscription_id,
						}
					},
					None,
				)
				.await?;

			// Find all periods that are either in the future or currently active
			let mut current_periods = current_periods
				.into_iter()
				.filter(|period| period.end > chrono::Utc::now() && period.start <= chrono::Utc::now())
				.collect::<Vec<_>>();

			// Sort them by the end date
			current_periods.sort_by(|a, b| a.end.cmp(&b.end));

			let start = current_periods.last().map(|period| period.end).unwrap_or(start);

			let end = start
				.checked_add_months(period_duration) // It's fine to use this function here since UTC doens't have daylight saving time transitions
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"invoice created_at is missing",
					))
				})?;

			for period in current_periods {
				if period.start > chrono::Utc::now() {
					break;
				}

				// Cancel the period on the provider
				match period.provider_id {
					Some(ProviderSubscriptionId::Stripe(id)) => {
						stripe::Subscription::update(
							stripe_client
								.client(super::StripeRequest::Invoice(StripeRequest::CancelSubscription(id.to_string())))
								.await
								.deref(),
							&id,
							stripe::UpdateSubscription {
								cancel_at_period_end: Some(true),
								..Default::default()
							},
						)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to update stripe subscription");
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::StripeError,
								"failed to update stripe subscription",
							))
						})?;
					}
					Some(ProviderSubscriptionId::Paypal(id)) => {
						let api_key = paypal_api::api_key(global).await.map_err(TransactionError::Custom)?;

						// https://developer.paypal.com/docs/api/subscriptions/v1/#subscriptions_cancel
						let response = global
							.http_client
							.post(format!("https://api.paypal.com/v1/billing/subscriptions/{id}/cancel"))
							.bearer_auth(&api_key)
							.json(&serde_json::json!({
								"reason": "Subscription canceled by gift"
							}))
							.send()
							.await
							.map_err(|e| {
								tracing::error!(error = %e, "failed to cancel paypal subscription");
								TransactionError::Custom(ApiError::internal_server_error(
									ApiErrorCode::PaypalError,
									"failed to cancel paypal subscription",
								))
							})?;

						if !response.status().is_success() {
							tracing::error!(status = %response.status(), "failed to cancel paypal subscription");
							return Err(TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::PaypalError,
								"failed to cancel paypal subscription",
							)));
						}
					}
					None => {}
				}
			}

			tx.insert_one(
				SubscriptionPeriod {
					id: SubscriptionPeriodId::new(),
					subscription_id,
					provider_id: None,
					start,
					end,
					product_id,
					is_trial: false,
					gifted_by: Some(customer_id),
					auto_renew: false,
					created_by: SubscriptionPeriodCreatedBy::Invoice {
						invoice_id: invoice.id.into(),
					},
					updated_at: chrono::Utc::now(),
					search_updated_at: None,
				},
				None,
			)
			.await?;

			return Ok(Some(subscription_id));
		}
		(
			None,
			Some(InvoiceMetadata::BoughtPeriod {
				user_id,
				start,
				end,
				product_id,
				subscription_product_id,
			}),
		) => {
			let sub_id = SubscriptionId {
				user_id,
				product_id: subscription_product_id,
			};

			// historical period
			tx.insert_one(
				SubscriptionPeriod {
					id: SubscriptionPeriodId::new(),
					subscription_id: sub_id,
					provider_id: None,
					start,
					end,
					product_id,
					is_trial: false,
					gifted_by: None,
					auto_renew: false,
					created_by: SubscriptionPeriodCreatedBy::Invoice {
						invoice_id: invoice.id.into(),
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
#[tracing::instrument(skip_all, name = "stripe::invoice::deleted")]
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
#[tracing::instrument(skip_all, name = "stripe::invoice::payment_failed")]
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

/// TODO: Delete after christmas gifter event
async fn handle_xmas_2024_gift(
	tx: &mut TransactionSession<'_, ApiError>,
	start: chrono::DateTime<chrono::Utc>,
	customer_id: UserId,
	subscription_product_id: SubscriptionProductId,
) -> TransactionResult<(), ApiError> {
	let xmas_event_start = chrono::Utc.with_ymd_and_hms(2024, 12, 14, 0, 0, 0).unwrap();
	let xmas_event_end = chrono::Utc.with_ymd_and_hms(2024, 12, 27, 0, 0, 0).unwrap();

	if start < xmas_event_start || start > xmas_event_end {
		return Ok(());
	}

	let Some(special_event_id) = tx
		.find_one(
			filter::filter! {
				SpecialEvent {
					name: "2024 X-MAS Gifter".to_owned(),
				}
			},
			None,
		)
		.await?
	else {
		tracing::warn!("could not find special event for xmas 2024 gift: {}", customer_id);
		return Ok(());
	};

	// We gift them a special event because they gifted a sub during the christmas
	// event
	let from = EntitlementEdgeKind::Subscription {
		subscription_id: SubscriptionId {
			user_id: customer_id,
			product_id: subscription_product_id,
		},
	};
	let managed_by = EntitlementEdgeManagedBy::SpecialEvent {
		special_event_id: special_event_id.id,
	};

	tx.delete(
		filter::filter! {
			EntitlementEdge {
				#[query(flatten, rename = "_id")]
				id: EntitlementEdgeId {
					#[query(serde)]
					from: &from,
					#[query(serde)]
					managed_by: Some(&managed_by),
				}
			}
		},
		None,
	)
	.await?;

	tx.insert_one(
		EntitlementEdge {
			id: EntitlementEdgeId {
				from,
				to: EntitlementEdgeKind::SpecialEvent {
					special_event_id: special_event_id.id,
				},
				managed_by: Some(managed_by),
			},
		},
		None,
	)
	.await?;

	Ok(())
}
