use mongodb::bson::DateTime;

use super::{invoice::InvoiceRef, ProductId};
use super::{InvoiceId, InvoiceLineItemId, ProductRef, SubscriptionId};
use crate::database::{Collection, Id, UserId};

pub type PurchaseId = Id<Purchase>;

// A purchase of a `Product`
// `Purchase` are always for products of kind `OneTimePurchase`
// Unlike the `Subscription`, a user can have multiple purchases of the same
// product.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Purchase {
	// This is a unique id for our system
	#[serde(rename = "_id")]
	pub id: PurchaseId,
	// The stripe id for the purchase (corrisponds to the `Product.id` field)
	pub product_id: ProductId,
	// Our internal id for the user who received the purchase
	pub user_id: UserId,
	// The invoice that created this purchase
	pub invoice: InvoiceRef,
	// If this item has been refunded
	pub refunded: bool,
}

impl Collection for Purchase {
	const COLLECTION_NAME: &'static str = "purchases";
}

// A subscription is a recurring purchase of a `Product` or multiple `Product`s
// In stripe a subscription is a special type of cron job that creates an
// invoice every billing cycle but also allows you to prorate the cost of the
// subscription if the user changes their plan.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscription {
	// The stripe id for the subscription
	#[serde(rename = "_id")]
	pub id: SubscriptionId,
	// The current active product ids for the subscription
	pub active_product_ids: Vec<ProductId>,
	// The user who received the subscription
	pub user_id: UserId,
	// Start time of the subscription
	pub start: DateTime,
	/// The current active period
	pub active_period: Option<SubscriptionPeriodId>,
	// Future periods that are scheduled to start
	pub scheduled_periods: Vec<SubscriptionScheduledPeriod>,
	// The status of this subscription
	pub standing: Option<SubscriptionStanding>,
	// Legacy PayPal subscription
	// In the past we used to use PayPal as our payment processor, we have since moved to Stripe.
	// However some subscriptions still exist that are paid through PayPal.
	// In this case we have a reference to the PayPal subscription that is managing this stripe subscription.
	pub paypal_subscription: Option<PayPalSubscription>,
	// If the subscription is active or not
	pub active: bool,
	// If this object is invalid, and the reason why our system thinks it is invalid
	pub invalid: Option<String>,
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
pub enum SubscriptionStanding {
	// Canceled by the user (ends at the end of the current period)
	Canceled = 0,
	// The subscription payment is overdue (active but payment has failed)
	Overdue = 1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionScheduledPeriod {
	/// The end time of the period will always be in the future
	pub end: DateTime,
	/// The items that will be in this period
	pub items: Vec<SubscriptionPeriodItem>,
}

impl Collection for Subscription {
	const COLLECTION_NAME: &'static str = "subscriptions";
}

pub type SubscriptionPeriodId = Id<SubscriptionPeriod>;

// Subscription Periods are the individual billing periods of a subscription
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriod {
	#[serde(rename = "_id")]
	pub id: SubscriptionPeriodId,
	// The start time of the period
	pub start: DateTime,
	// The time the period ended
	pub end: Option<DateTime>,
	// The time the period is projected to end (if the period is still active)
	// In some cases the period may end early, for example if the user cancels their subscription
	// Or if the subscription is refunded, or they change their plan.
	pub projected_end: DateTime,
	// The invoice that created this period
	pub invoice_id: Option<InvoiceId>,
	// If this period is enabled.
	pub state: SubscriptionPeriodState,
	// How this period was created (if none then it is a normal period)
	pub items: Vec<SubscriptionPeriodItem>,
	// If this period is a trial period and the reason why
	pub trial: Option<SubscriptionPeriodTrial>,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum SubscriptionPeriodState {
	Active = 0,
	Ended = 1,
	Refunded = 2,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriodTrial {
	/// The reason why this period is a trial, if any
	pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionPeriodItem {
	/// The product that this period is for
	pub product: ProductRef,
	/// The item in the invoice that created this period.
	pub invoice_item_id: Option<InvoiceLineItemId>,
	// A special kind of period, for example if the period was gifted
	// Or if we issued a free period for some reason
	pub special_kind: Option<SubscriptionPeriodSpecialKind>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", content = "data", rename_all = "snake_case")]
pub enum SubscriptionPeriodSpecialKind {
	// A gifted period
	Gift {
		/// The inventory item that this period was created from
		inventory_id: PurchaseInventoryItemId,
	},
	// A period created by the system
	System {
		reason: String,
	},
}

impl Collection for SubscriptionPeriod {
	const COLLECTION_NAME: &'static str = "subscription_periods";
}

pub type PurchaseInventoryItemId = Id<PurchaseInventoryItem>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PurchaseInventoryItem {
	#[serde(rename = "_id")]
	pub id: PurchaseInventoryItemId,
	/// The person who has this item in their inventory
	pub user_id: UserId,
	/// The product that was gifted
	pub products: ProductRef,
	/// The invoice that created the item
	pub invoice: InvoiceRef,
	/// State of the inventory item
	pub state: PurchaseInventoryState,
	/// Expire time of the item, if the item is not claimed by this time it will
	/// no longer be claimable.
	pub expires: Option<DateTime>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", content = "data", rename_all = "snake_case")]
pub enum PurchaseInventoryState {
	/// The item is currently unclaimed but
	UnclaimedGiftable {
		// A unique code that can be used to claim the item
		// When the code is none, it means the item's code has been revoked.
		code: Option<String>,
		// A set of users who can claim the item (if empty then anyone can claim the item, if they are not blacklisted)
		recipient_whitelist: Vec<UserId>,
		// A set of users who cannot claim the item (if empty only users in the whitelist can claim the item, unless that is
		// empty in which case anyone can claim the item)
		recipient_blacklist: Vec<UserId>,
		// The time the code expires
		expires: DateTime,
	},
	/// The item is currently unclaimed and cannot be gifted to another user
	UnclaimedNonGiftable,
	/// The item has been claimed by the user.
	Claimed,
	/// The item has been consumed by the user.
	Consumed,
	/// The item has been revoked by the system
	Revoked { reason: String },
	/// The item has been refunded
	Refunded,
}

impl Collection for PurchaseInventoryItem {
	const COLLECTION_NAME: &'static str = "purchase_inventory_items";
}
