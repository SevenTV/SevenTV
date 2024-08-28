use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
};

use shared::database::{
	product::{
		subscription::{ProviderSubscriptionId, SubscriptionPeriod},
		ProductId, SubscriptionProduct,
	},
	queries::{filter, update},
};
use stripe::Object;

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

/// Sets the subscription current period end to `ended_at`.
pub async fn deleted(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<(), ApiError> {
	let ended_at = subscription.ended_at.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;
	let ended_at = chrono::DateTime::from_timestamp(ended_at, 0).ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

	let subscription_id: ProviderSubscriptionId = subscription.id.into();

	tx.update_one(
		filter::filter! {
			SubscriptionPeriod {
				#[query(serde)]
				subscription_id: subscription_id,
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
		None,
	)
	.await?;

	Ok(())
}

/// Ends the current subscription period right away when the subscription products got removed from the subscription.
/// Otherwise, updates the current subscription period to include all updated subscription products.
pub async fn updated(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	event_created: chrono::DateTime<chrono::Utc>,
	subscription: stripe::Subscription,
	prev_attributes: HashMap<String, serde_json::Value>,
) -> TransactionResult<(), ApiError> {
	if !prev_attributes.contains_key("items") {
		// items didn't change, we don't have to handle this
		return Ok(());
	};

	let subscription_id: ProviderSubscriptionId = subscription.id.into();

	let product_ids = subscription
		.items
		.data
		.iter()
		.map(|item| Ok(item.price.as_ref().ok_or(ApiError::BAD_REQUEST)?.id().into()))
		.collect::<Result<HashSet<ProductId>, _>>()
		.map_err(TransactionError::custom)?;

	let products: HashMap<ProductId, SubscriptionProduct> = global
		.subscription_product_by_id_loader
		.load_many(product_ids.into_iter())
		.await
		.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

	if products.is_empty() {
		// end the current subscription period right now
		tx.update_one(
			filter::filter! {
				SubscriptionPeriod {
					#[query(serde)]
					subscription_id,
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
			None,
		)
		.await?;
	} else {
		// update current subscription period
		tx.update_one(
			filter::filter! {
				SubscriptionPeriod {
					#[query(serde)]
					subscription_id,
					#[query(selector = "lt")]
					start: chrono::Utc::now(),
					#[query(selector = "gt")]
					end: chrono::Utc::now(),
				}
			},
			update::update! {
				#[query(set)]
				SubscriptionPeriod {
					product_ids: products.into_keys().collect::<Vec<_>>(),
					updated_at: chrono::Utc::now(),
				}
			},
			None,
		)
		.await?;
	}

	Ok(())
}
