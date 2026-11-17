use std::collections::HashMap;

use chrono::{DateTime, Utc};
use shared::database::product::codes::RedeemCodeId;
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::{ProductId, StripeProductId, SubscriptionProductId};
use shared::database::user::UserId;

// Maximum number of recipients that can be gifted in a single checkout is set to 10
// because stripe metadata values are limited to 500 characters. Since user IDs are
// serialized as a comma separated list in a single metadata field, this cap keeps us
// well within that limit even for the longest supported id format.
// (Can also be adjusted if the id format changes).

pub const MAX_GIFT_RECIPIENTS: usize = 10;

mod comma_separated_recipients {
	use shared::database::user::UserId;

	pub fn serialize<S: serde::Serializer>(
		recipients: &[(UserId, u32)],
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		let joined = recipients
			.iter()
			.map(|(id, months)| format!("{id}:{months}"))
			.collect::<Vec<_>>()
			.join(",");
		serializer.serialize_str(&joined)
	}

	pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<(UserId, u32)>, D::Error> {
		let raw = String::deserialize(deserializer)?;
		raw.split(',')
			.filter(|s| !s.is_empty())
			.map(|pair| {
				let (id, months) = pair
					.split_once(':')
					.ok_or_else(|| serde::de::Error::custom(format!("invalid recipient entry: {pair}")))?;
				let id = id.parse::<UserId>().map_err(serde::de::Error::custom)?;
				let months = months.parse::<u32>().map_err(serde::de::Error::custom)?;
				Ok((id, months))
			})
			.collect()
	}
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(
	// tag = "KIND",
	untagged,
	rename_all = "SCREAMING_SNAKE_CASE",
	rename_all_fields = "SCREAMING_SNAKE_CASE"
)]
pub enum InvoiceMetadata {
	PaypalLegacy {
		paypal_id: String,
	},
	Gift {
		#[serde(with = "comma_separated_recipients")]
		recipients: Vec<(UserId, u32)>,
		customer_id: UserId,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		subscription_product_id: Option<SubscriptionProductId>,
		product_id: StripeProductId,
	},
	BoughtPeriod {
		user_id: UserId,
		start: DateTime<Utc>,
		end: DateTime<Utc>,
		subscription_product_id: SubscriptionProductId,
		product_id: StripeProductId,
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
	Redeem {
		user_id: UserId,
		redeem_code_id: RedeemCodeId,
	},
	Subscription,
	Gift,
	Setup,
	Pickems {
		user_id: UserId,
		product_id: ProductId,
	},
	CosmeticBundle {
		user_id: UserId,
		product_id: ProductId,
		special_event_id: SpecialEventId,
	},
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct CustomerMetadata {
	pub user_id: UserId,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub paypal_id: Option<String>,
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
