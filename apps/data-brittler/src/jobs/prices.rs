use std::str::FromStr;

use shared::database::product::{
	SubscriptionProduct, SubscriptionProductId, SubscriptionProductKind, SubscriptionProductVariant,
};

use super::subscriptions::{PAYPAL_MONTHLY, PAYPAL_YEARLY, STRIPE_MONTHLY, STRIPE_YEARLY};

pub const NEW_SUBSCRIPTION_PRODUCT_ID: &str = "01FEVKBBTGRAT7FCY276TNTJ4A";
pub const STRIPE_PRODUCT_ID: &str = "prod_KAlZaL8Yi2GRJA";

pub fn default_products() -> Vec<SubscriptionProduct> {
	vec![SubscriptionProduct {
		id: SubscriptionProductId::from_str(NEW_SUBSCRIPTION_PRODUCT_ID).unwrap(),
		provider_id: STRIPE_PRODUCT_ID.parse().unwrap(),
		variants: vec![
			SubscriptionProductVariant {
				id: STRIPE_MONTHLY.parse().unwrap(),
				active: true,
				kind: SubscriptionProductKind::Monthly,
				currency_prices: [(stripe::Currency::EUR, 399)].into_iter().collect(),
				paypal_id: Some(PAYPAL_MONTHLY.to_owned()),
			},
			SubscriptionProductVariant {
				id: STRIPE_YEARLY.parse().unwrap(),
				active: true,
				kind: SubscriptionProductKind::Yearly,
				currency_prices: [(stripe::Currency::EUR, 3999)].into_iter().collect(),
				paypal_id: Some(PAYPAL_YEARLY.to_owned()),
			},
		],
		default_variant_idx: 0,
		name: "7TV Subscription".to_string(),
		description: None,
		default_currency: stripe::Currency::EUR,
		benefits: super::subscriptions::benefits::sub_badges_benefits()
			.into_iter()
			.chain(super::subscriptions::benefits::sub_monthly_benefits())
			.map(|benefit| benefit.benefit)
			.collect(),
		updated_at: chrono::Utc::now(),
		search_updated_at: None,
	}]
}

