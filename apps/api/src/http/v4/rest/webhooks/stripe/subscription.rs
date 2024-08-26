use std::sync::Arc;

use shared::database::product::subscription::{SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId};

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

pub async fn created(
	_global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<(), ApiError> {
	if subscription.status == stripe::SubscriptionStatus::Incomplete {
		tracing::warn!(sub_id = ?subscription.id, "ignoring, subscription is incomplete");
		return Ok(());
	}

	let invoice_id = subscription
		.latest_invoice
		.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?
		.id()
		.into();

	let product_ids = subscription
		.items
		.data
		.into_iter()
		.map(|item| Ok(item.price.ok_or(ApiError::BAD_REQUEST)?.id.into()))
		.collect::<Result<_, ApiError>>()
		.map_err(TransactionError::custom)?;

	let period = SubscriptionPeriod {
		id: SubscriptionPeriodId::new(),
		subscription_id: subscription.id.into(),
		user_id: todo!(),
		start: chrono::DateTime::from_timestamp(subscription.current_period_start, 0)
			.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?,
		end: chrono::DateTime::from_timestamp(subscription.current_period_end, 0)
			.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?,
		is_trial: subscription.trial_end.is_some(),
		created_by: SubscriptionPeriodCreatedBy::Invoice { invoice_id },
		product_ids,
		updated_at: chrono::Utc::now(),
		search_updated_at: None,
	};

	tx.insert_one(period, None).await?;

	Ok(())
}

pub async fn deleted(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<(), ApiError> {
	// TODO: do we have to do something here?
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}

pub async fn updated(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	subscription: stripe::Subscription,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}
