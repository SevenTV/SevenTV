use std::collections::HashMap;
use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy,
};
use shared::database::product::{ProductId, SubscriptionProduct, SubscriptionProductVariant};
use shared::database::queries::{filter, update};
use shared::database::stripe_errors::{StripeError, StripeErrorId, StripeErrorKind};

use crate::global::Global;
use crate::http::egvault::metadata::{StripeMetadata, SubscriptionMetadata};
use crate::http::error::{ApiError, ApiErrorCode};
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

fn subscription_products(items: stripe::List<stripe::SubscriptionItem>) -> Result<Vec<ProductId>, ApiError> {
	items
		.data
		.into_iter()
		.map(|i| {
			Ok(ProductId::from(
				i.price
					.ok_or_else(|| ApiError::bad_request(ApiErrorCode::StripeError, "subscription item price is missing"))?
					.id,
			))
		})
		.collect()
}

/// Creates the subscription object in the database.
#[tracing::instrument(skip_all, name = "stripe::subscription::created")]
pub async fn created(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
	event_id: stripe::EventId,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	if subscription.metadata.is_empty() {
		tracing::warn!("subscription metadata is missing: {}", subscription.id);
		return Ok(None);
	}

	let items = subscription_products(subscription.items).map_err(TransactionError::Custom)?;

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
		// only accept subs with one product
		tracing::warn!("subscription has more than one product: {}", subscription.id);
		tx.insert_one(
			StripeError {
				id: StripeErrorId::new(),
				event_id,
				error_kind: StripeErrorKind::SubscriptionMultipleProducts,
			},
			None,
		)
		.await?;

		return Ok(None);
	}

	let metadata = SubscriptionMetadata::from_stripe(&subscription.metadata).map_err(|e| {
		tracing::error!(error = %e, "failed to deserialize metadata");
		TransactionError::Custom(ApiError::internal_server_error(
			ApiErrorCode::StripeError,
			"failed to deserialize metadata",
		))
	})?;

	let product = products.into_iter().next().unwrap();

	let sub_id = SubscriptionId {
		user_id: metadata.user_id,
		product_id: product.id,
	};

	Ok(Some(sub_id))
}

/// Sets the subscription current period end to `ended_at`.
#[tracing::instrument(skip_all, name = "stripe::subscription::deleted")]
pub async fn deleted(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let ended_at = subscription.ended_at.ok_or_else(|| {
		TransactionError::Custom(ApiError::bad_request(
			ApiErrorCode::StripeError,
			"subscription ended_at is missing",
		))
	})?;
	let ended_at = chrono::DateTime::from_timestamp(ended_at, 0).ok_or_else(|| {
		TransactionError::Custom(ApiError::bad_request(
			ApiErrorCode::StripeError,
			"subscription ended_at is missing",
		))
	})?;

	let subscription_id: ProviderSubscriptionId = subscription.id.into();

	let Some(period) = tx
		.find_one_and_update(
			filter::filter! {
				SubscriptionPeriod {
					#[query(serde)]
					provider_id: &subscription_id,
					#[query(selector = "lt")]
					start: chrono::Utc::now(),
					#[query(selector = "gt")]
					end: chrono::Utc::now(),
				}
			},
			update::update! {
				#[query(set)]
				SubscriptionPeriod {
					end: ended_at,
					updated_at: chrono::Utc::now(),
					search_updated_at: &None,
				}
			},
			FindOneAndUpdateOptions::builder()
				.return_document(ReturnDocument::After)
				.build(),
		)
		.await?
	else {
		return Ok(None);
	};

	Ok(Some(period.subscription_id))
}

/// Ends the current subscription period right away when the subscription
/// products got removed from the subscription. Otherwise, updates the current
/// subscription period to include all updated subscription products.
#[tracing::instrument(skip_all, name = "stripe::subscription::updated")]
pub async fn updated(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	event_id: stripe::EventId,
	event_created: chrono::DateTime<chrono::Utc>,
	subscription: stripe::Subscription,
	prev_attributes: HashMap<String, serde_json::Value>,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let items = subscription_products(subscription.items).map_err(TransactionError::Custom)?;

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

	let items_changed = prev_attributes.contains_key("items");

	match (items_changed, products.len()) {
		(true, 0) => {
			// product was removed from the subscription
			// end the current period right away

			let Some(period) = tx
				.find_one_and_update(
					filter::filter! {
						SubscriptionPeriod {
							#[query(serde)]
							provider_id: Some(ProviderSubscriptionId::Stripe(subscription.id.into())),
							#[query(selector = "lt")]
							start: chrono::Utc::now(),
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
						}
					},
					update::update! {
						#[query(set)]
						SubscriptionPeriod {
							end: event_created,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					FindOneAndUpdateOptions::builder()
						.return_document(ReturnDocument::After)
						.build(),
				)
				.await?
			else {
				return Ok(None);
			};

			return Ok(Some(period.subscription_id));
		}
		(true, 1) => {
			tx.insert_one(
				StripeError {
					id: StripeErrorId::new(),
					event_id,
					error_kind: StripeErrorKind::SubscriptionItemChanged,
				},
				None,
			)
			.await?;
		}
		(true, _) => {
			// n > 1
			// the subscription has more than one product now
			tx.insert_one(
				StripeError {
					id: StripeErrorId::new(),
					event_id,
					error_kind: StripeErrorKind::SubscriptionMultipleProducts,
				},
				None,
			)
			.await?;
		}
		(false, 1) => {
			// nothing changed, still one product
			// update subscription

			let Some(invoice) = subscription.latest_invoice else {
				return Ok(None);
			};

			// update cancel_at_period_end of the current subscription period
			let Some(period) = tx
				.find_one_and_update(
					filter::filter! {
						SubscriptionPeriod {
							#[query(serde)]
							provider_id: Some(ProviderSubscriptionId::Stripe(subscription.id.into())),
							gifted_by: &None,
							#[query(serde)]
							created_by: SubscriptionPeriodCreatedBy::Invoice {
								invoice_id: invoice.id().into(),
							},
						}
					},
					update::update! {
						#[query(set)]
						SubscriptionPeriod {
							auto_renew: !subscription.cancel_at_period_end,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					FindOneAndUpdateOptions::builder()
						.return_document(ReturnDocument::After)
						.build(),
				)
				.await?
			else {
				return Ok(None);
			};

			return Ok(Some(period.subscription_id));
		}
		(false, _) => {
			// n == 0 || n > 1
			// nothing changed, still more than one product or zero products
		}
	}

	Ok(None)
}
