use super::codes::{GiftCodeId, RedeemCodeId};
use super::{CustomerId, InvoiceId, ProductId, SubscriptionId};
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, MongoCollection};

/// Only required for paypal subscriptions since they can't save any metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "paypal_subscriptions")]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct PaypalSubscription {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: String,
	/// The user that receives the subscription benefits
	pub user_id: UserId,
	/// Set if there is a stripe customer for this customer already
	pub stripe_customer_id: Option<CustomerId>,
	pub product_id: ProductId,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum ProviderSubscriptionId {
	Stripe(SubscriptionId),
	Paypal(String),
}

impl ToString for ProviderSubscriptionId {
	fn to_string(&self) -> String {
		match self {
			Self::Stripe(id) => id.to_string(),
			Self::Paypal(id) => id.clone(),
		}
	}
}

impl From<SubscriptionId> for ProviderSubscriptionId {
	fn from(id: SubscriptionId) -> Self {
		Self::Stripe(id)
	}
}

impl From<stripe::SubscriptionId> for ProviderSubscriptionId {
	fn from(value: stripe::SubscriptionId) -> Self {
		Self::Stripe(value.into())
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscription_periods")]
#[mongo(index(fields(user_id = 1)))]
#[mongo(index(fields(subscription_id = 1)))]
#[mongo(index(fields(start = 1)))]
#[mongo(index(fields(end = 1)))]
#[mongo(index(fields(product_ids = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionPeriodId,
	pub subscription_id: ProviderSubscriptionId,
	pub user_id: UserId,
	#[serde(with = "crate::database::serde")]
	pub start: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub end: chrono::DateTime<chrono::Utc>,
	pub is_trial: bool,
	pub created_by: SubscriptionPeriodCreatedBy,
	pub product_ids: Vec<ProductId>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type SubscriptionPeriodId = Id<SubscriptionPeriod>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum SubscriptionPeriodCreatedBy {
	RedeemCode {
		redeem_code_id: RedeemCodeId,
	},
	GiftCode {
		gift_code_id: GiftCodeId,
	},
	Invoice {
		invoice_id: InvoiceId,
	},
	System {
		reason: Option<String>,
	},
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<SubscriptionPeriod>(),
	]
}
