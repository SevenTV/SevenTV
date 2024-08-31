use std::sync::Arc;

use shared::database::product::subscription::{ProviderSubscriptionId, Subscription, SubscriptionPeriod};
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
) -> TransactionResult<(), ApiError> {
	let subscription_id = ProviderSubscriptionId::Paypal(subscription.id);

	let now = chrono::Utc::now();

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
				ended_at: Some(now),
				updated_at: now,
			}
		},
		None,
	)
	.await?;

	tx.update_one(
		filter::filter! {
			SubscriptionPeriod {
				#[query(serde)]
				subscription_id,
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
		None,
	)
	.await?;

	Ok(())
}
