use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use shared::database::product::{
	Product, SubscriptionProduct, SubscriptionProductId, SubscriptionProductKind, SubscriptionProductVariant,
};
use shared::database::MongoCollection;
use stripe::{Recurring, RecurringInterval};

use super::subscriptions::{PAYPAL_MONTHLY, PAYPAL_YEARLY, STRIPE_MONTHLY, STRIPE_YEARLY};
use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub const NEW_PRODUCT_ID: &str = "01FEVKBBTGRAT7FCY276TNTJ4A";

pub struct PricesJob {
	global: Arc<Global>,
	subscription_product: SubscriptionProduct,
}

impl Job for PricesJob {
	type T = types::Price;

	const NAME: &'static str = "transfer_prices";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping products and subscription_products collection");
			Product::collection(global.target_db()).drop().await?;
			let indexes = Product::indexes();
			if !indexes.is_empty() {
				Product::collection(global.target_db()).create_indexes(indexes).await?;
			}

			SubscriptionProduct::collection(global.target_db()).drop().await?;
			let indexes = SubscriptionProduct::indexes();
			if !indexes.is_empty() {
				SubscriptionProduct::collection(global.target_db())
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self {
			global,
			subscription_product: SubscriptionProduct {
				id: SubscriptionProductId::from_str(NEW_PRODUCT_ID).unwrap(),
				variants: vec![],
				default_variant_idx: 0,
				name: "7TV Subscription".to_string(),
				description: None,
				default_currency: stripe::Currency::USD,
				benefits: vec![],
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			},
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

		let price_id = match stripe::PriceId::from_str(&price.provider_id) {
			Ok(price_id) => price_id,
			Err(e) => {
				return outcome.with_error(error::Error::InvalidStripeId(e));
			}
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
				} => SubscriptionProductKind::Monthly,
				Recurring {
					interval: RecurringInterval::Year,
					..
				} => SubscriptionProductKind::Yearly,
				_ => {
					return outcome.with_error(error::Error::InvalidRecurringInterval(recurring.interval));
				}
			};

			let paypal_id = if price_id.as_str() == STRIPE_MONTHLY {
				Some(PAYPAL_MONTHLY.to_string())
			} else if price_id.as_str() == STRIPE_YEARLY {
				Some(PAYPAL_YEARLY.to_string())
			} else {
				None
			};

			self.subscription_product.default_currency = currency;
			self.subscription_product.variants.push(SubscriptionProductVariant {
				id: price_id.into(),
				gift_id: None,
				kind,
				currency_prices,
				paypal_id,
			});
		} else {
			match Product::collection(self.global.target_db())
				.insert_one(Product {
					id: price_id.into(),
					name: product.name.unwrap_or_default(),
					extends_subscription: None,
					description: None,
					default_currency: currency,
					currency_prices,
					created_at: chrono::Utc::now(),
					updated_at: chrono::Utc::now(),
					search_updated_at: None,
				})
				.await
			{
				Ok(_) => outcome.inserted_rows += 1,
				Err(e) => outcome.errors.push(e.into()),
			}
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing prices job");

		match SubscriptionProduct::collection(self.global.target_db())
			.insert_one(&self.subscription_product)
			.await
		{
			Ok(_) => ProcessOutcome::default(),
			Err(e) => ProcessOutcome::error(e),
		}
	}
}
