use std::collections::HashMap;

use shared::database::product::codes::RedeemCodeId;
use shared::database::product::SubscriptionProductId;
use shared::database::user::UserId;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(
	tag = "KIND",
	rename_all = "SCREAMING_SNAKE_CASE",
	rename_all_fields = "SCREAMING_SNAKE_CASE"
)]
pub enum InvoiceMetadata {
	Gift {
		user_id: UserId,
		customer_id: UserId,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		subscription_product_id: Option<SubscriptionProductId>,
	},
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct SubscriptionMetadata {
	pub user_id: UserId,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub customer_id: Option<UserId>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(
	rename_all = "SCREAMING_SNAKE_CASE",
	tag = "KIND",
	rename_all_fields = "SCREAMING_SNAKE_CASE"
)]
pub enum CheckoutSessionMetadata {
	Redeem { user_id: UserId, redeem_code_id: RedeemCodeId },
	Subscription,
	Gift,
	Setup,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct CustomerMetadata {
	pub user_id: UserId,
}

pub trait StripeMetadata: serde::Serialize + serde::de::DeserializeOwned {
	fn from_stripe(metadata: &HashMap<String, String>) -> Result<Self, serde_json::Error> {
		let value = serde_json::to_value(metadata)?;
		serde_json::from_value(value)
	}

	fn to_stripe(&self) -> HashMap<String, String> {
		let value = serde_json::to_value(self).expect("failed to serialize metadata");
		serde_json::from_value(value).expect("failed to deserialize to hashmap")
	}
}

impl StripeMetadata for InvoiceMetadata {}
impl StripeMetadata for SubscriptionMetadata {}
impl StripeMetadata for CheckoutSessionMetadata {}
impl StripeMetadata for CustomerMetadata {}
