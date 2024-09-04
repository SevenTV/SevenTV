use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy,
};
use shared::database::product::{ProductId, SubscriptionProduct, SubscriptionProductVariant};
use shared::database::queries::{filter, update};
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

fn subscription_products(items: stripe::List<stripe::SubscriptionItem>) -> Result<Vec<ProductId>, ApiError> {
	items
		.data
		.into_iter()
		.map(|i| Ok(ProductId::from(i.price.ok_or(ApiError::BAD_REQUEST)?.id)))
		.collect()
}

/// Creates the subscription object in the database.
pub async fn created(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let items = subscription_products(subscription.items).map_err(TransactionError::custom)?;

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
		return Ok(None);
	}

	let user_id = subscription
		.metadata
		.get("USER_ID")
		.and_then(|i| UserId::from_str(i).ok())
		.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

	let product = products.into_iter().next().unwrap();

	let sub_id = SubscriptionId {
		user_id,
		product_id: product.id,
	};

	Ok(Some(sub_id))
}

/// Sets the subscription current period end to `ended_at`.
pub async fn deleted(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let ended_at = subscription.ended_at.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;
	let ended_at = chrono::DateTime::from_timestamp(ended_at, 0).ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

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
pub async fn updated(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	event_created: chrono::DateTime<chrono::Utc>,
	subscription: stripe::Subscription,
	prev_attributes: HashMap<String, serde_json::Value>,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let items = subscription_products(subscription.items).map_err(TransactionError::custom)?;

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
			// product was swapped with another product

			// let user_id = subscription
			// 	.metadata
			// 	.get("USER_ID")
			// 	.and_then(|i| UserId::from_str(i).ok())
			// 	.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

			// let old_product = prev_attributes
			// 	.get("items")
			// 	.and_then(|v| v.get("data"))
			// 	.and_then(|v| v.get(0))
			// 	.and_then(|v| v.get("price"))
			// 	.and_then(|v| v.get("id"))
			// 	.and_then(|v| v.as_str())
			// 	.and_then(|s| ProductId::from_str(s).ok())
			// 	.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;
		}
		(true, _) => {
			// n > 1
			// the subscription has more than one product now
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
							#[query(selector = "lt")]
							start: chrono::Utc::now(),
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
							#[query(serde)]
							created_by: SubscriptionPeriodCreatedBy::Invoice {
								invoice_id: invoice.id().into(),
								cancel_at_period_end: false,
							},
						}
					},
					update::update! {
						#[query(set)]
						SubscriptionPeriod {
							#[query(serde)]
							created_by: SubscriptionPeriodCreatedBy::Invoice {
								invoice_id: invoice.id().into(),
								cancel_at_period_end: subscription.cancel_at_period_end,
							},
							updated_at: chrono::Utc::now(),
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
