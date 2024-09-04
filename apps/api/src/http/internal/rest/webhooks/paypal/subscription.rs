use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::product::subscription::{ProviderSubscriptionId, Subscription, SubscriptionId, SubscriptionPeriod, SubscriptionState};
use shared::database::queries::{filter, update};

use super::types;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionResult, TransactionSession};

/// Ends the current period right away.
///
/// Called for `BILLING.SUBSCRIPTION.CANCELLED` and
/// `BILLING.SUBSCRIPTION.SUSPENDED`
pub async fn cancelled(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: types::Subscription,
) -> TransactionResult<Option<SubscriptionId>, ApiError> {
	let subscription_id = ProviderSubscriptionId::Paypal(subscription.id);

	let now = chrono::Utc::now();

	let Some(period) = tx
		.find_one_and_update(
			filter::filter! {
				SubscriptionPeriod {
					#[query(serde)]
					provider_id: subscription_id,
					#[query(selector = "lt")]
					start: now,
					#[query(selector = "gt")]
					end: now,
				}
			},
			update::update! {
				#[query(set)]
				SubscriptionPeriod {
					end: now,
					updated_at: now,
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
				updated_at: now,
			}
		},
		None,
	)
	.await?;

	Ok(Some(period.subscription_id))
}
