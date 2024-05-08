use std::{str::FromStr, sync::Arc};

use shared::database::{Collection, GatewayProvider, Price};

use crate::{error::Error, global::Global, paypal, types};

use super::{Job, ProcessOutcome};

pub struct PricesJob {
	global: Arc<Global>,
	stripe_client: stripe::Client,
}

impl Job for PricesJob {
	const NAME: &'static str = "transfer_prices";

	type T = types::Price;

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating prices collection");
			Price::collection(global.target_db()).drop(None).await?;
		}

		let stripe_client = stripe::Client::new(&global.config().stripe_key);
		Ok(Self { global, stripe_client })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.egvault_source_db().collection("prices")
	}

	async fn process(&mut self, price: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let (provider, data) = match price.provider {
			types::PriceProvider::Paypal => {
				match paypal::Plan::query_by_id(&price.provider_id, &self.global.config().paypal).await {
					Ok(mut pp_plan) => {
						let first_billing_cycle = pp_plan.billing_cycles.remove(0);
						if first_billing_cycle.tenure_type != paypal::BillingCycleTenureType::Regular {
							panic!("unsupported paypal subscription type");
						}
						let mut unit_amount = f64::from_str(&first_billing_cycle.pricing_scheme.fixed_price.value).unwrap();
						unit_amount *= 100.0;
						let unit_amount = unit_amount as i64;
						let data = stripe::Price {
							active: Some(pp_plan.status == paypal::PlanStatus::Active),
							billing_scheme: Some(stripe::PriceBillingScheme::PerUnit),
							created: Some(pp_plan.create_time.timestamp()),
							currency: Some(first_billing_cycle.pricing_scheme.fixed_price.currency_code),
							livemode: Some(true),
							nickname: pp_plan.description,
							recurring: Some(stripe::Recurring {
								interval: first_billing_cycle.frequency.interval_unit.into(),
								interval_count: first_billing_cycle.frequency.interval_count as u64,
								usage_type: stripe::RecurringUsageType::Licensed,
								..Default::default()
							}),
							type_: Some(stripe::PriceType::Recurring),
							unit_amount: Some(unit_amount),
							unit_amount_decimal: Some(unit_amount.to_string()),
							..Default::default()
						};
						(GatewayProvider::Paypal, data)
					}
					Err(e) => {
						outcome.errors.push(Error::Paypal(e));
						return outcome;
					}
				}
			}
			types::PriceProvider::Stripe => {
				let Ok(stripe_id) = stripe::PriceId::from_str(&price.provider_id) else {
					outcome.errors.push(Error::InvalidStripeId(price.provider_id));
					return outcome;
				};
				match stripe::Price::retrieve(&self.stripe_client, &stripe_id, &["product"]).await {
					Ok(data) => (GatewayProvider::Stripe, data),
					Err(e) => {
						outcome.errors.push(e.into());
						return outcome;
					}
				}
			}
		};

		let price = Price {
			id: price.id.into(),
			rank: price.template_index as i16,
			provider,
			provider_id: price.provider_id,
			data,
			..Default::default()
		};

		if let Err(e) = Price::collection(self.global.target_db()).insert_one(price, None).await {
			outcome.errors.push(e.into());
		}

		outcome
	}
}
