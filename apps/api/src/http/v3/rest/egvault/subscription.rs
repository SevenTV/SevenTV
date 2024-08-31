use std::sync::Arc;

use axum::extract::State;
use axum::{Extension, Json};
use chrono::TimeDelta;
use futures::TryStreamExt;
use hyper::StatusCode;
use shared::database::product::subscription::{
	ProviderSubscriptionId, Subscription, SubscriptionPeriod, SubscriptionPeriodCreatedBy,
};
use shared::database::product::SubscriptionProductKind;
use shared::database::queries::filter;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::rest::types::{self, SubscriptionCycleStatus};
use crate::http::v3::rest::users::TargetUser;

#[derive(Debug, serde::Serialize)]
pub struct SubscriptionResponse {
	pub active: bool,
	pub age: u32,
	pub renew: bool,
	pub subscription: Option<types::Subscription>,
}

pub async fn subscription(
	State(global): State<Arc<Global>>,
	Path(target): Path<TargetUser>,
	auth_session: Option<Extension<AuthSession>>,
) -> Result<Json<SubscriptionResponse>, ApiError> {
	let user = match target {
		TargetUser::Me => auth_session.ok_or(ApiError::UNAUTHORIZED)?.user_id(),
		TargetUser::Other(id) => id,
	};

	let periods: Vec<_> = SubscriptionPeriod::collection(&global.db)
		.find(filter::filter! {
			SubscriptionPeriod {
				user_id: user,
			}
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to find subscription period");
			ApiError::INTERNAL_SERVER_ERROR
		})?
		.try_collect()
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to collect subscription periods");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

	let Some(active_period) = periods
		.iter()
		.find(|p| p.start < chrono::Utc::now() && p.end > chrono::Utc::now())
	else {
		return Ok(Json(SubscriptionResponse {
			active: false,
			age: 0,
			renew: false,
			subscription: None,
		}));
	};

	let subscription = match &active_period.subscription_id {
		Some(id) => global
			.subscription_by_id_loader
			.load(id.clone())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?,
		None => None,
	};

	let Some(product_id) = active_period.product_ids.iter().next() else {
		return Err(ApiError::new_const(StatusCode::NOT_FOUND, "subscription product not found"));
	};

	let product = global
		.subscription_product_by_id_loader
		.load(product_id.clone())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "subscription product not found"))?;

	let age = periods
		.iter()
		.map(|p| p.end.signed_duration_since(p.start).max(TimeDelta::zero()))
		.sum::<chrono::TimeDelta>()
		.num_days();

	let provider = active_period.subscription_id.as_ref().map(|id| match id {
		ProviderSubscriptionId::Stripe(_) => types::Provider::Stripe,
		ProviderSubscriptionId::Paypal(_) => types::Provider::Paypal,
	});

	let customer_id = match active_period.created_by {
		SubscriptionPeriodCreatedBy::Gift { gifter, .. } => gifter,
		_ => active_period.user_id,
	}
	.to_string();

	let status = match &subscription {
		Some(Subscription { ended_at: Some(_), .. }) => SubscriptionCycleStatus::Ended,
		Some(Subscription {
			cancel_at_period_end: true,
			..
		}) => SubscriptionCycleStatus::Canceled,
		_ => SubscriptionCycleStatus::Ongoing,
	};

	let internal = matches!(active_period.created_by, SubscriptionPeriodCreatedBy::System { .. });

	let unit = match product.kind {
		SubscriptionProductKind::Monthly => types::SubscriptionCycleUnit::Month,
		SubscriptionProductKind::Yearly => types::SubscriptionCycleUnit::Year,
	};

	let renew = subscription.as_ref().map(|s| !s.cancel_at_period_end).unwrap_or(false);

	let ended_at = subscription.as_ref().and_then(|s| s.ended_at);

	Ok(Json(SubscriptionResponse {
		active: true,
		age: age.try_into().unwrap_or(0),
		renew,
		subscription: Some(types::Subscription {
			id: active_period.id.to_string(),
			provider,
			product_id: product.id.to_string(),
			plan: product.id.to_string(),
			seats: 1,
			subscriber_id: active_period.user_id.to_string(),
			customer_id,
			started_at: subscription.as_ref().map(|s| s.created_at).unwrap_or(active_period.start),
			ended_at,
			cycle: types::SubscriptionCycle {
				timestamp: active_period.end,
				unit,
				value: 1,
				status,
				internal,
				pending: false,
				trial_end: subscription.as_ref().and_then(|s| s.trial_end),
			},
			renew,
			end_at: ended_at,
		}),
	}))
}
