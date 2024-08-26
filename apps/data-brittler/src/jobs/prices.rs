use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use shared::database::product::{Product, SubscriptionKind, SubscriptionProduct};
use shared::database::MongoCollection;
use stripe::{Recurring, RecurringInterval};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct PricesJob {
	global: Arc<Global>,
	products: Vec<Product>,
	subscription_products: Vec<SubscriptionProduct>,
}

impl Job for PricesJob {
	type T = types::Price;

	const NAME: &'static str = "transfer_prices";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping products collection");
			Product::collection(global.target_db()).drop().await?;
			let indexes = Product::indexes();
			if !indexes.is_empty() {
				Product::collection(global.target_db()).create_indexes(indexes).await?;
			}
		}

		Ok(Self {
			global,
			products: vec![],
			subscription_products: vec![],
		})
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.egvault_source_db().collection("prices"))
	}

	async fn process(&mut self, price: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		if price.provider != types::GatewayProvider::Stripe {
			// skip
			return outcome;
		}

		let Ok(price_id) = stripe::PriceId::from_str(&price.provider_id) else {
			outcome.errors.push(error::Error::InvalidStripeId(price.provider_id));
			return outcome;
		};

		let price =
			match stripe::Price::retrieve(self.global.stripe_client(), &price_id, &["product", "currency_options"]).await {
				Ok(price) => price,
				Err(e) => {
					outcome.errors.push(e.into());
					return outcome;
				}
			};
		let product = price.product.and_then(|p| p.into_object()).expect("no product found");

		let mut currency_prices = HashMap::new();

		let currency = price.currency.expect("no currency found");
		let unit_amount = price.unit_amount.expect("no unit amount found");
		currency_prices.insert(currency, unit_amount.max(0) as i32);

		let currency_options = price.currency_options.expect("no currency options found");
		for (currency, unit_amount) in currency_options
			.into_iter()
			.filter_map(|(c, o)| o.unit_amount.map(|a| (c, a)))
		{
			currency_prices.insert(currency, unit_amount.max(0) as i32);
		}

		if let Some(recurring) = price.recurring {
			let kind = match recurring {
				Recurring {
					interval: RecurringInterval::Month,
					..
				} => SubscriptionKind::Monthly,
				Recurring {
					interval: RecurringInterval::Year,
					..
				} => SubscriptionKind::Yearly,
				_ => {
					return outcome.with_error(error::Error::InvalidRecurringInterval(recurring.interval));
				}
			};

			self.subscription_products.push(SubscriptionProduct {
				id: price_id.into(),
				name: product.name.unwrap_or_default(),
				description: None,
				kind,
				benefits: vec![],
				default_currency: currency,
				currency_prices,
				created_at: chrono::Utc::now(),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			});
		} else {
			self.products.push(Product {
				id: price_id.into(),
				name: product.name.unwrap_or_default(),
				description: None,
				default_currency: currency,
				currency_prices,
				created_at: chrono::Utc::now(),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			});
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing prices job");

		let mut outcome = ProcessOutcome::default();

		match Product::collection(self.global.target_db())
			.insert_many(&self.products)
			.with_options(mongodb::options::InsertManyOptions::builder().ordered(false).build())
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.products.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		match SubscriptionProduct::collection(self.global.target_db())
			.insert_many(&self.subscription_products)
			.with_options(mongodb::options::InsertManyOptions::builder().ordered(false).build())
			.await
		{
			Ok(res) => {
				outcome.inserted_rows += res.inserted_ids.len() as u64;
				if res.inserted_ids.len() != self.products.len() {
					outcome.errors.push(error::Error::InsertMany);
				}
			}
			Err(e) => outcome.errors.push(e.into()),
		}

		outcome
	}
}
