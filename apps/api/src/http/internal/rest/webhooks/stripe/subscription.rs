use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, UpdateOptions};
use shared::database::product::subscription::{
	ProviderSubscriptionId, Subscription, SubscriptionId, SubscriptionPeriod, SubscriptionState,
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
) -> TransactionResult<(), ApiError> {
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
		return Ok(());
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

	tx.update_one(
		filter::filter! {
			Subscription {
				#[query(rename = "_id", serde)]
				id: &sub_id,
			}
		},
		update::update! {
			#[query(set_on_insert)]
			Subscription {
				id: sub_id.clone(),
				state: SubscriptionState::Active,
				updated_at: chrono::Utc::now(),
			}
		},
		UpdateOptions::builder().upsert(true).build(),
	)
	.await?;

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
		return Ok(());
	};

	tx.update_one(
		filter::filter! {
			Subscription {
				#[query(rename = "_id", serde)]
				id: period.subscription_id,
			}
		},
		update::update! {
			#[query(set)]
			Subscription {
				#[query(serde)]
				state: SubscriptionState::Ended,
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}

/// Ends the current subscription period right away when the subscription
/// products got removed from the subscription. Otherwise, updates the current
/// subscription period to include all updated subscription products.
pub async fn updated(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	event_created: chrono::DateTime<chrono::Utc>,
	subscription: stripe::Subscription,
	prev_attributes: HashMap<String, serde_json::Value>,
) -> TransactionResult<(), ApiError> {
	// if !prev_attributes.contains_key("items") {
	// 	// items didn't change, we don't have to handle this
	// 	return Ok(());
	// };

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

	if products.is_empty() {
		// all products were removed from the subscription
		// end the current subscription period right now

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
			return Ok(());
		};

		tx.update_one(
			filter::filter! {
				Subscription {
					#[query(rename = "_id", serde)]
					id: &period.subscription_id,
				}
			},
			update::update! {
				#[query(set)]
				Subscription {
					#[query(serde)]
					state: SubscriptionState::Ended,
					updated_at: chrono::Utc::now(),
				}
			},
			None,
		)
		.await?;
	} else {
		if products.len() != 1 {
			// only accept subs with one product
			return Ok(());
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

		if prev_attributes.contains_key("items") {
			// items changed

			// TODO: handle this case
		} else {
			// items didn't change, we don't have to handle this

			let new_state = if subscription.ended_at.is_some() {
				SubscriptionState::Ended
			} else if subscription.cancel_at_period_end {
				SubscriptionState::CancelAtEnd
			} else {
				SubscriptionState::Active
			};

			tx.update_one(
				filter::filter! {
					Subscription {
						#[query(rename = "_id", serde)]
						id: &sub_id,
					}
				},
				update::update! {
					#[query(set)]
					Subscription {
						#[query(serde)]
						state: new_state,
						updated_at: chrono::Utc::now(),
					}
				},
				None,
			)
			.await?;
		}
	}

	Ok(())
}
