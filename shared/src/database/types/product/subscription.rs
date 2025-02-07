use std::fmt::Display;
use std::str::FromStr;

use super::codes::RedeemCodeId;
use super::{InvoiceId, StripeProductId, StripeSubscriptionId, SubscriptionProductId};
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, IdFromStrError, MongoCollection};
use crate::typesense::types::TypesenseType;

#[derive(
	Debug,
	Clone,
	Copy,
	serde::Serialize,
	serde::Deserialize,
	Hash,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	async_graphql::SimpleObject,
)]
pub struct SubscriptionId {
	pub user_id: UserId,
	pub product_id: SubscriptionProductId,
}

impl Display for SubscriptionId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.user_id, self.product_id)
	}
}

impl FromStr for SubscriptionId {
	type Err = IdFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (user_id, product_id) = s.split_once(':').ok_or(IdFromStrError::InvalidLength(s.len()))?;

		Ok(Self {
			user_id: user_id.parse()?,
			product_id: product_id.parse()?,
		})
	}
}

// All subscriptions that ever existed, not only active ones
// This is only used to save data about a subscription that could also be
// retrieved from Stripe or PayPal It is used to avoid sending requests to
// Stripe or PayPal every time someone queries data about a subscription
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscriptions")]
#[mongo(index(fields("_id.user_id" = 1, "_id.product_id" = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::product::subscription::Subscription")]
#[serde(deny_unknown_fields)]
pub struct Subscription {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionId,
	pub state: SubscriptionState,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum SubscriptionState {
	Active = 0,
	CancelAtEnd = 1,
	Ended = 2,
}

impl TypesenseType for SubscriptionState {
	fn typesense_type() -> crate::typesense::types::FieldType {
		crate::typesense::types::FieldType::Int32
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum ProviderSubscriptionId {
	Stripe(StripeSubscriptionId),
	Paypal(String),
}

impl Display for ProviderSubscriptionId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Stripe(id) => write!(f, "{id}"),
			Self::Paypal(id) => write!(f, "{id}"),
		}
	}
}

impl From<StripeSubscriptionId> for ProviderSubscriptionId {
	fn from(id: StripeSubscriptionId) -> Self {
		Self::Stripe(id)
	}
}

impl From<stripe::SubscriptionId> for ProviderSubscriptionId {
	fn from(value: stripe::SubscriptionId) -> Self {
		Self::Stripe(value.into())
	}
}

/// Current or past periods of a subscription (not future)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscription_periods")]
#[mongo(index(fields("subscription_id.user_id" = 1, start = 1, end = 1)))]
#[mongo(index(fields(subscription_id = 1)))]
#[mongo(index(fields(product_ids = 1)))]
#[mongo(index(fields(provider_id = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::product::subscription::SubscriptionPeriod")]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionPeriodId,
	pub subscription_id: SubscriptionId,
	pub provider_id: Option<ProviderSubscriptionId>,
	pub product_id: StripeProductId,
	#[serde(with = "crate::database::serde")]
	pub start: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub end: chrono::DateTime<chrono::Utc>,
	pub is_trial: bool,
	pub auto_renew: bool,
	pub gifted_by: Option<UserId>,
	pub created_by: SubscriptionPeriodCreatedBy,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type SubscriptionPeriodId = Id<SubscriptionPeriod>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "type")]
pub enum SubscriptionPeriodCreatedBy {
	RedeemCode { redeem_code_id: RedeemCodeId },
	Invoice { invoice_id: InvoiceId },
	System { reason: Option<String> },
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<SubscriptionPeriod>(),
		MongoGenericCollection::new::<Subscription>(),
	]
}
