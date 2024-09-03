use std::future::IntoFuture;
use std::str::FromStr;
use std::sync::Arc;

use mongodb::options::InsertManyOptions;
use shared::database::product::subscription::{
	ProviderSubscriptionId, Subscription, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy
};
use shared::database::product::SubscriptionProductId;
use shared::database::MongoCollection;

use super::prices::NEW_PRODUCT_ID;
use super::{Job, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, SubscriptionProvider};

pub const PAYPAL_YEARLY: &'static str = "P-9P108407878214437MDOSLGA";
pub const PAYPAL_MONTHLY: &'static str = "P-0RN164482K927302CMDOSJJA";
pub const STRIPE_YEARLY: &'static str = "price_1JWQ2RCHxsWbK3R3a6emz76a";
pub const STRIPE_MONTHLY: &'static str = "price_1JWQ2QCHxsWbK3R31cZkaocV";

pub struct SubscriptionsJob {
	global: Arc<Global>,
	subscriptions: Vec<Subscription>,
	periods: Vec<SubscriptionPeriod>,
}

impl Job for SubscriptionsJob {
	type T = types::Subscription;

	const NAME: &'static str = "transfer_subscriptions";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			Subscription::collection(global.target_db()).drop().await?;
			let indexes = Subscription::indexes();
			if !indexes.is_empty() {
				Subscription::collection(global.target_db()).create_indexes(indexes).await?;
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
			subscriptions: vec![],
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
			product_id: SubscriptionProductId::from_str(NEW_PRODUCT_ID).unwrap(),
		};

		self.subscriptions.push(Subscription {
			id: sub_id.clone(),
			state: subscription.cycle.status.into(),
			updated_at: chrono::Utc::now(),
		});

		self.periods.push(SubscriptionPeriod {
			id: subscription.id.into(),
			provider_id: Some(provider_id),
			subscription_id: sub_id,
			start: subscription.started_at.into_chrono(),
			end: subscription
				.ended_at
				.or(subscription.cycle.timestamp)
				.map(|t| t.into_chrono())
				.unwrap_or_else(chrono::Utc::now),
			is_trial: false,
			created_by: SubscriptionPeriodCreatedBy::System {
				reason: Some("Old subscription".to_string()),
			},
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		});

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing subscriptions job");

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let subscriptions = Subscription::collection(self.global.target_db());
		let periods = SubscriptionPeriod::collection(self.global.target_db());

		let res = tokio::join!(
			subscriptions
				.insert_many(&self.subscriptions)
				.with_options(insert_options.clone())
				.into_future(),
			periods
				.insert_many(&self.periods)
				.with_options(insert_options.clone())
				.into_future(),
		);
		let res = vec![res.0, res.1]
			.into_iter()
			.zip(vec![self.subscriptions.len(), self.periods.len()]);

		for (res, len) in res {
			match res {
				Ok(res) => {
					outcome.inserted_rows += res.inserted_ids.len() as u64;
					if res.inserted_ids.len() != len {
						outcome.errors.push(error::Error::InsertMany);
					}
				}
				Err(e) => outcome.errors.push(e.into()),
			}
		}

		outcome
	}
}
