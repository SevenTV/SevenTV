use std::str::FromStr;
use std::sync::Arc;

use shared::database::{Collection, Subscription};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct SubscriptionsJob {
	global: Arc<Global>,
	subscriptions: Vec<Subscription>,
}

impl Job for SubscriptionsJob {
	type T = types::Subscription;

	const NAME: &'static str = "transfer_stripe";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping subscriptions");
			Subscription::collection(global.target_db()).drop(None).await?;
		}

		Ok(Self {
			global,
			subscriptions: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.egvault_source_db().collection("subscriptions")
	}

	async fn process(&mut self, sub: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let (Some(provider_id), Some(cycle_timestamp), types::SubscriptionProvider::Stripe) =
			(sub.provider_id, sub.cycle.timestamp, sub.provider)
		else {
			// skip
			return outcome;
		};

		let Ok(provider_id) = stripe::SubscriptionId::from_str(&provider_id) else {
			outcome.errors.push(error::Error::InvalidStripeId(provider_id));
			return outcome;
		};
		let Ok(product_id) = stripe::ProductId::from_str(&sub.product_id) else {
			outcome.errors.push(error::Error::InvalidStripeId(sub.product_id));
			return outcome;
		};
		let Ok(price_id) = stripe::PriceId::from_str(&sub.plan_id) else {
			outcome.errors.push(error::Error::InvalidStripeId(sub.plan_id));
			return outcome;
		};

		// let (state, standing) = match sub.cycle.status {
		// 	types::SubscriptionCycleStatus::Ongoing => (SubscriptionState::Active, None),
		// 	types::SubscriptionCycleStatus::Ended => (SubscriptionState::Ended, None),
		// 	types::SubscriptionCycleStatus::Canceled => (SubscriptionState::Active,
		// Some(SubscriptionStanding::Canceled)), };

		// let end = match sub.cycle.unit {
		// 	types::SubscriptionCycleUnit::Year =>
		// cycle_timestamp.into_chrono().checked_add_months(Months::new(12)),
		// 	types::SubscriptionCycleUnit::Month =>
		// cycle_timestamp.into_chrono().checked_add_months(Months::new(1)),
		// 	types::SubscriptionCycleUnit::Day =>
		// cycle_timestamp.into_chrono().checked_add_days(Days::new(1)), }
		// .expect("failed to add duration");

		// self.subscriptions.push(Subscription {
		// 	id: provider_id,
		// 	product_id,
		// 	user_id: sub.subscriber_id.into(),
		// 	start: sub.started_at.into(),
		// 	periods: vec![SubscriptionPeriod {
		// 		end: end.into(),
		// 		special_kind: None,
		// 		invoice_id: None,
		// 		enabled: true,
		// 		product_price_id: price_id,
		// 	}],
		// 	scheduled_periods: vec![],
		// 	standing,
		// 	paypal_subscription: None,
		// 	state,
		// });

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing stripe job");

		let mut outcome = ProcessOutcome::default();

		match Subscription::collection(self.global.target_db())
			.insert_many(&self.subscriptions, None)
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.subscriptions.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
