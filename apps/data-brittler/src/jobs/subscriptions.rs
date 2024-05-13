use std::sync::Arc;

use mongodb::bson::oid::ObjectId;
use shared::database::{
	Collection, GatewayProvider, PriceId, Purchase, PurchaseData, RedeemCode, RedeemCodeId, RedeemCodeRecipient,
	RedeemCodeType, SubscriptionStatus,
};

use crate::{
	error::Error,
	global::Global,
	types::{self, SubscriptionCycleStatus},
};

use super::{Job, ProcessOutcome};

pub struct SubscriptionsJob {
	global: Arc<Global>,
	purchases: Vec<Purchase>,
	nnys_live_code: RedeemCodeId,
	unknown_code: RedeemCodeId,
}

impl Job for SubscriptionsJob {
	const NAME: &'static str = "transfer_subscriptions";

	type T = types::Subscription;

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping purchases and redeem_codes collection");
			Purchase::collection(global.target_db()).drop(None).await?;
			RedeemCode::collection(global.target_db()).drop(None).await?;
		}

		// create nnys.live code
		let nnys_live_code = RedeemCodeId::new();
		RedeemCode::collection(global.target_db())
			.insert_one(
				RedeemCode {
					id: nnys_live_code,
					name: "nnys.live".into(),
					description: None,
					enabled: false,
					recipient: RedeemCodeRecipient::Anyone {
						code: "NNYS2023".to_string(),
						remaining_uses: None,
					},
					redeem_type: RedeemCodeType::ExNihilo {
						price_ids: vec![PriceId::from_object_id(
							ObjectId::parse_str("60ffa1a1e45b451d1f7d34a1").unwrap(),
						)],
					},
				},
				None,
			)
			.await?;

		// create unknown code
		let unknown_code = RedeemCodeId::new();
		RedeemCode::collection(global.target_db())
			.insert_one(
				RedeemCode {
					id: unknown_code,
					name: "Unknown Code".into(),
					description: Some("A placeholder for any code that was redeemed before the migration.".to_string()),
					enabled: false,
					recipient: RedeemCodeRecipient::Anyone {
						code: "unknown".to_string(),
						remaining_uses: None,
					},
					redeem_type: RedeemCodeType::ExNihilo { price_ids: vec![] },
				},
				None,
			)
			.await?;

		Ok(Self {
			global,
			purchases: vec![],
			nnys_live_code,
			unknown_code,
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.egvault_source_db().collection("subscriptions")
	}

	async fn process(&mut self, subscription: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let data = match subscription.provider {
			types::SubscriptionProvider::Stripe | types::SubscriptionProvider::Paypal => {
				let provider = match subscription.provider {
					types::SubscriptionProvider::Stripe => GatewayProvider::Stripe,
					types::SubscriptionProvider::Paypal => GatewayProvider::Paypal,
					_ => unreachable!(),
				};
				let price: Option<types::Price> = self
					.global
					.egvault_source_db()
					.collection("prices")
					.find_one(
						mongodb::bson::doc! {
							"provider_id": &subscription.plan_id
						},
						None,
					)
					.await
					.unwrap();

				let price = match price {
					Some(price) => price,
					None => {
						outcome.errors.push(Error::PriceNotFound(subscription.plan_id));
						return outcome;
					}
				};

				PurchaseData::Subscription {
					provider,
					provider_id: subscription.provider_id.clone(),
					status: match subscription.cycle.status {
						SubscriptionCycleStatus::Ongoing => SubscriptionStatus::Active,
						SubscriptionCycleStatus::Ended => SubscriptionStatus::Canceled,
						SubscriptionCycleStatus::Canceled => SubscriptionStatus::Canceled,
					},
					prices: vec![price.id.into()],
					cancel_at_period_end: subscription.cycle.status == SubscriptionCycleStatus::Canceled,
					cancel_at: None,
					created: subscription.id.timestamp(),
					ended_at: subscription.ended_at.map(|t| t.into_chrono().into()),
					trial_end: subscription.cycle.trial_end_at.map(|t| t.into_chrono().into()),
				}
			}
			types::SubscriptionProvider::NnysLive => PurchaseData::RedeemCode { id: self.nnys_live_code },
			types::SubscriptionProvider::RedeemCode => PurchaseData::RedeemCode { id: self.unknown_code },
			// ignore subscriptions without a provider (only 10, all ended or canceled)
			types::SubscriptionProvider::None => return outcome,
		};

		let purchase = Purchase {
			id: subscription.id.into(),
			user_id: subscription.subscriber_id.into(),
			data,
		};

		self.purchases.push(purchase);

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing subscriptions job");

		let mut outcome = ProcessOutcome::default();

		let len = self.purchases.len();

		match Purchase::collection(self.global.target_db())
			.insert_many(self.purchases, None)
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != len {
					outcome.errors.push(Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
