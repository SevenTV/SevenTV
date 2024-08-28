use std::sync::Arc;

use shared::database::{
	product::subscription::{ProviderSubscriptionId, SubscriptionPeriod},
	queries::{filter, update},
};

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionResult, TransactionSession},
};

use super::types;

/// Ends the current period right away.
///
/// Called for `BILLING.SUBSCRIPTION.CANCELLED` and `BILLING.SUBSCRIPTION.SUSPENDED`
pub async fn cancelled(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	subscription: types::Subscription,
) -> TransactionResult<(), ApiError> {
	tx.update_one(
		filter::filter! {
			SubscriptionPeriod {
				#[query(serde)]
				subscription_id: ProviderSubscriptionId::Paypal(subscription.id),
				#[query(selector = "lt")]
				start: chrono::Utc::now(),
				#[query(selector = "gt")]
				end: chrono::Utc::now(),
			}
		},
		update::update! {
			#[query(set)]
			SubscriptionPeriod {
				end: chrono::Utc::now(),
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	Ok(())
}

/// There is no invoice for this payment because it only gets created when the payment succeeds.
pub async fn payment_failed(
	_global: &Arc<Global>,
	_tx: TransactionSession<'_, ApiError>,
	_subscription: types::Subscription,
) -> TransactionResult<(), ApiError> {
	Ok(())
}
