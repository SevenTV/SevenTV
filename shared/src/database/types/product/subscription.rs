use std::fmt::Display;
use std::str::FromStr;

use super::codes::RedeemCodeId;
use super::{CustomerId, InvoiceId, PaymentIntentId, ProductId, SubscriptionId};
use crate::database::types::MongoGenericCollection;
use crate::database::user::UserId;
use crate::database::{Id, MongoCollection};

/// All subscriptions that ever existed, not only active ones
/// This is only used to save data about a subscription that could also be
/// retrieved from Stripe or PayPal It is used to avoid sending requests to
/// Stripe or PayPal every time someone queries data about a subscription
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscriptions")]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct Subscription {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: ProviderSubscriptionId,
	/// The user that receives the subscription benefits
	pub user_id: UserId,
	/// Set if there is a stripe customer for this customer already
	/// always set for stripe subscriptions
	pub stripe_customer_id: Option<CustomerId>,
	pub product_ids: Vec<ProductId>,
	pub cancel_at_period_end: bool,
	pub trial_end: Option<chrono::DateTime<chrono::Utc>>,
	pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Subscription {
	pub fn from_stripe(sub: stripe::Subscription) -> Option<Self> {
		let created_at = match sub.ended_at {
			Some(t) => chrono::DateTime::from_timestamp(t, 0)?,
			None => chrono::Utc::now(),
		};

		let user_id = sub.metadata.get("USER_ID").and_then(|i| UserId::from_str(i).ok())?;

		Some(Self {
			id: sub.id.into(),
			user_id,
			stripe_customer_id: Some(sub.customer.id().into()),
			product_ids: sub
				.items
				.data
				.into_iter()
				.map(|i| Ok(i.price.ok_or(())?.id.into()))
				.collect::<Result<_, ()>>()
				.ok()?,
			cancel_at_period_end: sub.cancel_at_period_end,
			trial_end: match sub.trial_end {
				Some(t) => Some(chrono::DateTime::from_timestamp(t, 0)?),
				None => None,
			},
			ended_at: match sub.ended_at {
				Some(t) => Some(chrono::DateTime::from_timestamp(t, 0)?),
				None => None,
			},
			created_at,
			updated_at: created_at,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum ProviderSubscriptionId {
	Stripe(SubscriptionId),
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

/// Current or past periods of a subscription (not future)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MongoCollection)]
#[mongo(collection_name = "subscription_periods")]
#[mongo(index(fields(user_id = 1, start = 1, end = 1)))]
#[mongo(index(fields(subscription_id = 1)))]
#[mongo(index(fields(product_ids = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(_id = 1, updated_at = -1)))]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionPeriodId,
	/// None for gifted and system subscriptions
	pub subscription_id: Option<ProviderSubscriptionId>,
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
	RedeemCode { redeem_code_id: RedeemCodeId },
	Invoice { invoice_id: InvoiceId },
	Gift { gifter: UserId, payment: PaymentIntentId },
	System { reason: Option<String> },
}

pub(super) fn collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<SubscriptionPeriod>(),
		MongoGenericCollection::new::<Subscription>(),
	]
}
