use std::str::FromStr;
use std::sync::Arc;

use fnv::FnvHashMap;
use shared::database::{Collection, Product, ProductKind, ProductPrice};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct PricesJob {
	global: Arc<Global>,
	products: FnvHashMap<stripe::ProductId, Product>,
}

impl Job for PricesJob {
	type T = types::Price;

	const NAME: &'static str = "transfer_prices";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping products collection");
			Product::collection(global.target_db()).drop(None).await?;
		}

		Ok(Self {
			global,
			products: FnvHashMap::default(),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.egvault_source_db().collection("prices")
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

		let price = match stripe::Price::retrieve(self.global.stripe_client(), &price_id, &[]).await {
			Ok(price) => price,
			Err(e) => {
				outcome.errors.push(e.into());
				return outcome;
			}
		};
		let product = price.product.expect("no product found");

		let kind = if price.recurring.is_some() {
			ProductKind::Subscription
		} else {
			ProductKind::OneTimePurchase
		};

		// let entry = self.products.entry(product.id()).or_insert(Product {
		// 	id: product.id(),
		// 	kind,
		// 	prices: vec![],
		// });
		// entry.prices.push(ProductPrice {
		// 	id: price_id,
		// });

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing prices job");

		let mut outcome = ProcessOutcome::default();

		match Product::collection(self.global.target_db())
			.insert_many(self.products.values(), None)
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
