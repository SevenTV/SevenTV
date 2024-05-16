use mongodb::bson::DateTime;

use crate::database::{Id, UserId};

use super::invoice::InvoiceRef;

pub type PurchaseId = Id<Purchase>;

// A purchase of a `Product`
// `Purchase` are always for products of kind `OneTimePurchase`
// Unlike the `Subscription`, a user can have multiple purchases of the same product.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Purchase {
	// This is a unique id for our system
	#[serde(rename = "_id")]
	pub id: PurchaseId,
	// The stripe id for the purchase (corrisponds to the `Product.id` field)
	pub product_id: stripe::ProductId,
	// Our internal id for the user who received the purchase
	pub user_id: UserId,
	// The invoice that created this purchase
	pub invoice: InvoiceRef,
	// If this item has been refunded
	pub refuned: bool,
}

// A subscription to a `Product`
// `Subscription` are always for products of kind `Subscription`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscription {
	// The stripe id for the subscription
	#[serde(rename = "_id")]
	pub id: stripe::SubscriptionId,
	// Product id for the subscription
	pub product_id: stripe::ProductId,
	// The user who received the subscription
	pub user_id: UserId,
	// Start time of the subscription
	pub start: DateTime,
	// The periods of this subscription (these cannot overlap and are in order)
	pub periods: Vec<SubscriptionPeriod>,
	// Future periods that are scheduled to start
	pub scheduled_periods: Vec<SubscriptionScheduledPeriod>,
	// The status of this subscription
	pub standing: Option<SubscriptionStanding>,
	// Legacy PayPal subscription
	pub paypal_subscription: Option<PayPalSubscription>,
	// If the subscription is active or not
	pub state: SubscriptionState,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayPalSubscription {
	pub id: String,
	pub state: Option<PayPalSubscriptionState>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PayPalSubscriptionState {
	Payment(String),
	Invoice(String),
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum SubscriptionState {
	// The subscription is active
	Active = 0,
	// The subscription is paused
	Ended = 1,
	// The subscription is in an invalid state. (however it is still active)
	Invalid = 2,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum SubscriptionStanding {
	// Cancelled by the user (ends at the end of the current period)
	Cancelled = 0,
	// The subscription payment is overdue (active but payment has failed)
	Overdue = 1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionScheduledPeriod {
	// The end time of the period will always be in the future
	pub end: DateTime,
	// How this period was created
	pub kind: SubscriptionPeriodKind,
	// The price id for the period
	pub product_price_id: String,
}

// Subscription Periods only have a single end time because they are always contiguous
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	// The end time of the period (may be in the future if the period is ongoing)
	pub end: DateTime,
	// How this period was created (if none then it is a normal period)
	pub kind: Option<SubscriptionPeriodKind>,
	// The invoice that created this period
	pub invoice_id: String,
	// If this period is enabled.
	pub enabled: bool,
	// Price id for the period
	pub product_price_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", content = "data", rename_all = "snake_case")]
pub enum SubscriptionPeriodKind {
	// A trial period
	Trial {
		// The reason for the trial
		reason: Option<String>,
	},
	// A gifted period
	Gift {
		// The user who gifted the period
		gifter_id: UserId,
		// The invoice that created the gift
		invoice: InvoiceRef,
	},
	// A period created by the system
	System {
		reason: String,
	},
}
