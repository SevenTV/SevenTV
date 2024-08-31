use std::{collections::HashMap, sync::Arc};

use shared::database::{
	product::{
		subscription::{ProviderSubscriptionId, Subscription, SubscriptionPeriod},
		ProductId, SubscriptionProduct,
	},
	queries::{filter, update},
};

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

/// Creates the subscription object in the database.
pub async fn created(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<(), ApiError> {
	let subscription = Subscription::from_stripe(subscription).ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

	tx.insert_one(subscription, None).await?;

	Ok(())
}

/// Sets the subscription current period end to `ended_at`.
pub async fn deleted(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<(), ApiError> {
	let ended_at = subscription.ended_at.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;
	let ended_at = chrono::DateTime::from_timestamp(ended_at, 0).ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

	let subscription_id: ProviderSubscriptionId = subscription.id.into();

	tx.update_one(
		filter::filter! {
			Subscription {
				#[query(rename = "_id", serde)]
				id: &subscription_id,
			}
		},
		update::update! {
			#[query(set)]
			Subscription {
				ended_at: Some(ended_at),
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

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

	let subscription = Subscription::from_stripe(subscription).ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

	tx.update_one(
		filter::filter! {
			Subscription {
				#[query(rename = "_id", serde)]
				id: &subscription.id,
			}
		},
		update::update! {
			#[query(set)]
			Subscription {
				stripe_customer_id: subscription.stripe_customer_id,
				product_ids: &subscription.product_ids,
				cancel_at_period_end: subscription.cancel_at_period_end,
				trial_end: subscription.trial_end,
				ended_at: subscription.ended_at,
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	let products: HashMap<ProductId, SubscriptionProduct> = global
		.subscription_product_by_id_loader
		.load_many(subscription.product_ids.into_iter())
		.await
		.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

	if products.is_empty() {
		// end the current subscription period right now
		tx.update_one(
			filter::filter! {
				SubscriptionPeriod {
					#[query(serde)]
					subscription_id: subscription.id,
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
					subscription_id: subscription.id,
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
