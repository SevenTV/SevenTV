use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use chrono::Months;
use futures::StreamExt;
use shared::database::product::subscription::{
	ProviderSubscriptionId, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy,
};
use shared::database::product::{ProductId, SubscriptionProductId};

use super::prices::NEW_SUBSCRIPTION_PRODUCT_ID;
use super::{JobOutcome, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, SubscriptionCycleUnit, SubscriptionProvider};

pub mod benefits;

pub const PAYPAL_YEARLY: &str = "P-9P108407878214437MDOSLGA";
pub const PAYPAL_MONTHLY: &str = "P-0RN164482K927302CMDOSJJA";
pub const STRIPE_YEARLY: &str = "price_1JWQ2RCHxsWbK3R3a6emz76a";
pub const STRIPE_MONTHLY: &str = "price_1JWQ2QCHxsWbK3R31cZkaocV";

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub periods: &'a mut Vec<SubscriptionPeriod>,
}

#[tracing::instrument(skip_all, name = "subscriptions")]
pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("subscriptions");

	let RunInput { global, periods } = input;

	let mut cursor = global
		.egvault_source_db
		.collection::<types::Subscription>("subscriptions")
		.find(bson::doc! {})
		.await
		.context("query")?;

	while let Some(subscription) = cursor.next().await {
		match subscription {
			Ok(subscription) => {
				outcome += process(ProcessInput { periods, subscription });
				outcome.processed_documents += 1;
			}
			Err(e) => {
				tracing::error!("{:#}", e);
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	periods: &'a mut Vec<SubscriptionPeriod>,
	subscription: types::Subscription,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let outcome = ProcessOutcome::default();

	let ProcessInput { periods, subscription } = input;

	// skip one sub that doesnt have a plan id
	let Some(plan_id) = subscription.plan_id else {
		return outcome;
	};

	let Some(subscription_id) = subscription.provider_id else {
		return outcome;
	};

	let provider_id = match subscription.provider {
		SubscriptionProvider::Stripe => match stripe::SubscriptionId::from_str(&subscription_id) {
			Ok(id) => id.into(),
			Err(e) => return outcome.with_error(error::Error::InvalidStripeId(e)),
		},
		SubscriptionProvider::Paypal => ProviderSubscriptionId::Paypal(subscription_id),
		_ => return outcome,
	};

	let sub_id = SubscriptionId {
		user_id: subscription.subscriber_id.into(),
		product_id: SubscriptionProductId::from_str(NEW_SUBSCRIPTION_PRODUCT_ID).unwrap(),
	};

	let start = subscription.started_at.into_chrono();

	let end = match subscription.ended_at.map(|t| t.into_chrono()).or_else(|| {
		let unit = match subscription.cycle.unit {
			SubscriptionCycleUnit::Month | SubscriptionCycleUnit::Day => Months::new(1),
			SubscriptionCycleUnit::Year => Months::new(12),
		};

		subscription.cycle.timestamp?.into_chrono().checked_add_months(unit)
	}) {
		Some(end) => end,
		None => {
			return outcome.with_error(error::Error::InvalidSubscriptionCycle);
		}
	};

	if start > end {
		return outcome.with_error(error::Error::InvalidSubscriptionCycle);
	}

	periods.push(SubscriptionPeriod {
		id: subscription.id.into(),
		provider_id: Some(provider_id),
		product_id: match subscription.provider {
			SubscriptionProvider::Stripe => ProductId::from_str(&plan_id).unwrap(),
			SubscriptionProvider::Paypal => match plan_id.as_str() {
				PAYPAL_MONTHLY => ProductId::from_str(STRIPE_MONTHLY).unwrap(),
				PAYPAL_YEARLY => ProductId::from_str(STRIPE_YEARLY).unwrap(),
				_ => ProductId::from_str(STRIPE_MONTHLY).unwrap(),
			},
			_ => ProductId::from_str(STRIPE_MONTHLY).unwrap(),
		},
		subscription_id: sub_id,
		start,
		end,
		is_trial: subscription.cycle.trial_end_at.is_some(),
		created_by: SubscriptionPeriodCreatedBy::System {
			reason: Some("Data migration job".to_string()),
		},
		updated_at: chrono::Utc::now(),
		search_updated_at: None,
	});

	outcome
}
