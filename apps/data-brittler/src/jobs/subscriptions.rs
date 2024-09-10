use std::collections::HashMap;
use std::future::IntoFuture;
use std::str::FromStr;
use std::sync::Arc;

use chrono::Months;
use fnv::FnvHashSet;
use mongodb::options::InsertManyOptions;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::product::subscription::{
	ProviderSubscriptionId, Subscription, SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy,
};
use shared::database::product::{ProductId, SubscriptionProductId};
use shared::database::MongoCollection;

use super::prices::NEW_PRODUCT_ID;
use super::{Job, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, SubscriptionCycleStatus, SubscriptionCycleUnit, SubscriptionProvider};

pub mod benefits;

pub const PAYPAL_YEARLY: &str = "P-9P108407878214437MDOSLGA";
pub const PAYPAL_MONTHLY: &str = "P-0RN164482K927302CMDOSJJA";
pub const STRIPE_YEARLY: &str = "price_1JWQ2RCHxsWbK3R3a6emz76a";
pub const STRIPE_MONTHLY: &str = "price_1JWQ2QCHxsWbK3R31cZkaocV";

pub struct SubscriptionsJob {
	global: Arc<Global>,
	subscriptions: HashMap<SubscriptionId, Subscription>,
	periods: Vec<SubscriptionPeriod>,
	edges: FnvHashSet<EntitlementEdge>,
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
			subscriptions: Default::default(),
			periods: vec![],
			edges: FnvHashSet::default(),
		})
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.egvault_source_db().collection("subscriptions"))
	}

	async fn process(&mut self, subscription: Self::T) -> ProcessOutcome {
		let outcome = ProcessOutcome::default();

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
			product_id: SubscriptionProductId::from_str(NEW_PRODUCT_ID).unwrap(),
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

		if start < chrono::Utc::now() && end > chrono::Utc::now() {
			self.edges.insert(EntitlementEdge {
				id: EntitlementEdgeId {
					from: EntitlementEdgeKind::User { user_id: sub_id.user_id },
					to: EntitlementEdgeKind::Subscription { subscription_id: sub_id },
					managed_by: Some(EntitlementEdgeManagedBy::Subscription { subscription_id: sub_id }),
				},
			});
		}

		let sub = self.subscriptions.entry(sub_id).or_insert(Subscription {
			id: sub_id,
			updated_at: chrono::Utc::now(),
			created_at: subscription.id.timestamp().to_chrono(),
			search_updated_at: None,
			ended_at: None,
			state: subscription.cycle.status.into(),
		});

		sub.created_at = sub.created_at.min(subscription.id.timestamp().to_chrono());
		sub.state = subscription.cycle.status.into();
		sub.ended_at = match &subscription.cycle.status {
			SubscriptionCycleStatus::Ended | SubscriptionCycleStatus::Canceled => Some(sub.ended_at.unwrap_or(end).max(end)),
			_ => None,
		};

		self.periods.push(SubscriptionPeriod {
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

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing subscriptions job");

		// In case of truncate = true, we have to wait for the entitlements job to
		// finish truncating. Otherwise we will loose the edges here.
		if self.global.config().should_run_entitlements() && self.global.config().truncate {
			self.global.entitlement_job_token().cancelled().await;
		}

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let subscriptions = Subscription::collection(self.global.target_db());
		let periods = SubscriptionPeriod::collection(self.global.target_db());
		let edges = EntitlementEdge::collection(self.global.target_db());

		let res = tokio::join!(
			subscriptions
				.insert_many(self.subscriptions.values())
				.with_options(insert_options.clone())
				.into_future(),
			periods
				.insert_many(&self.periods)
				.with_options(insert_options.clone())
				.into_future(),
			edges
				.insert_many(&self.edges)
				.with_options(insert_options.clone())
				.into_future(),
		);
		let res =
			vec![res.0, res.1, res.2]
				.into_iter()
				.zip(vec![self.subscriptions.len(), self.periods.len(), self.edges.len()]);

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
