use std::str::FromStr;
use std::sync::Arc;

use shared::database::product::subscription::{PaypalSubscription, ProviderSubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy};
use shared::database::product::ProductId;
use shared::database::MongoCollection;

use super::{Job, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, SubscriptionProvider};

const PAYPAL_YEARLY: &'static str = "P-9P108407878214437MDOSLGA";
const PAYPAL_MONTHLY: &'static str = "P-0RN164482K927302CMDOSJJA";
const STRIPE_YEARLY: &'static str = "price_1JWQ2RCHxsWbK3R3a6emz76a";
const STRIPE_MONTHLY: &'static str = "price_1JWQ2QCHxsWbK3R31cZkaocV";

pub struct SubscriptionsJob {
	global: Arc<Global>,
	paypal_subscriptions: Vec<PaypalSubscription>,
	periods: Vec<SubscriptionPeriod>,
}

impl Job for SubscriptionsJob {
	type T = types::Subscription;

	const NAME: &'static str = "transfer_subscriptions";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			PaypalSubscription::collection(global.target_db()).drop().await?;
			let indexes = PaypalSubscription::indexes();
			if !indexes.is_empty() {
				PaypalSubscription::collection(global.target_db())
					.create_indexes(indexes)
					.await?;
			}

			SubscriptionPeriod::collection(global.target_db()).drop().await?;
			let indexes = SubscriptionPeriod::indexes();
			if !indexes.is_empty() {
				SubscriptionPeriod::collection(global.target_db())
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self {
			global,
			paypal_subscriptions: vec![],
			periods: vec![],
		})
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.egvault_source_db().collection("subscriptions"))
	}

	async fn process(&mut self, subscription: Self::T) -> ProcessOutcome {
		let outcome = ProcessOutcome::default();

		let Some(subscription_id) = subscription.provider_id else {
			return outcome;
		};

		let price_id = match subscription.plan_id.as_ref() {
			PAYPAL_MONTHLY => stripe::PriceId::from_str(STRIPE_MONTHLY).ok(),
			PAYPAL_YEARLY => stripe::PriceId::from_str(STRIPE_YEARLY).ok(),
			_ => None,
		};
		let Some(product_id) = price_id.map(ProductId::from) else {
			return outcome;
		};

		if subscription.provider == SubscriptionProvider::Paypal {
			self.paypal_subscriptions.push(PaypalSubscription {
				id: subscription_id.clone(),
				user_id: subscription.subscriber_id.into(),
				stripe_customer_id: None,
				product_id: product_id.clone(),
				created_at: subscription.id.timestamp().to_chrono(),
				updated_at: chrono::Utc::now(),
			});
		}

		let subscription_id = match subscription.provider {
			SubscriptionProvider::Stripe => match stripe::SubscriptionId::from_str(&subscription_id) {
				Ok(id) => id.into(),
				Err(e) => return outcome.with_error(error::Error::InvalidStripeId(e)),
			},
			SubscriptionProvider::Paypal => ProviderSubscriptionId::Paypal(subscription_id),
			_ => return outcome,
		};

		self.periods.push(SubscriptionPeriod {
			id: subscription.id.into(),
			subscription_id,
			user_id: subscription.subscriber_id.into(),
			start: subscription.started_at.into_chrono(),
			end: subscription
				.ended_at
				.or(subscription.cycle.timestamp)
				.map(|t| t.into_chrono())
				.unwrap_or_else(chrono::Utc::now),
			is_trial: false,
			created_by: SubscriptionPeriodCreatedBy::System { reason: Some("Old subscription".to_string()) },
			product_ids: vec![product_id],
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		});

		outcome
	}
}
