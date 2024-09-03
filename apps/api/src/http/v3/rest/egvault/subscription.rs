use std::sync::Arc;

use axum::extract::State;
use axum::{Extension, Json};
use futures::TryStreamExt;
use hyper::StatusCode;
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionState,
};
use shared::database::product::SubscriptionProduct;
use shared::database::queries::filter;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::rest::types::{self, SubscriptionCycleUnit};
use crate::http::v3::rest::users::TargetUser;
use crate::sub_refresh_job;

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
				#[query(flatten)]
				subscription_id: SubscriptionId {
					user_id: user,
				},
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
		.cloned()
	else {
		return Ok(Json(SubscriptionResponse {
			active: false,
			age: 0,
			renew: false,
			subscription: None,
		}));
	};

	let subscription = global
		.subscription_by_id_loader
		.load(active_period.subscription_id.clone())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	let periods: Vec<_> = periods
		.into_iter()
		.filter(|p| p.subscription_id == active_period.subscription_id)
		.collect();

	let product: SubscriptionProduct = global
		.subscription_product_by_id_loader
		.load(subscription.id.product_id)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "subscription product not found"))?;

	let age = sub_refresh_job::sub_age_days(&periods);

	let provider = active_period.provider_id.as_ref().map(|id| match id {
		ProviderSubscriptionId::Stripe(_) => types::Provider::Stripe,
		ProviderSubscriptionId::Paypal(_) => types::Provider::Paypal,
	});

	let customer_id = match active_period.created_by {
		SubscriptionPeriodCreatedBy::Gift { gifter, .. } => gifter,
		_ => active_period.subscription_id.user_id,
	}
	.to_string();

	let internal = matches!(active_period.created_by, SubscriptionPeriodCreatedBy::System { .. });

	// TODO: figure out how to get the unit
	let unit = Some(SubscriptionCycleUnit::Month);

	let started_at = periods
		.iter()
		.min_by_key(|p| p.start)
		.map(|p| p.start)
		.unwrap_or(active_period.start);

	let end_at = periods
		.iter()
		.max_by_key(|p| p.end)
		.map(|p| p.end)
		.unwrap_or(active_period.end);

	let trial_end = active_period.is_trial.then_some(active_period.end);

	let renew = subscription.state != SubscriptionState::CancelAtEnd;

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
			subscriber_id: subscription.id.user_id.to_string(),
			customer_id,
			started_at,
			ended_at: (subscription.state == SubscriptionState::Ended).then_some(end_at),
			cycle: types::SubscriptionCycle {
				timestamp: active_period.end,
				unit,
				value: 1,
				status: subscription.state.into(),
				internal,
				pending: false,
				trial_end,
			},
			renew,
			end_at,
		}),
	}))
}
