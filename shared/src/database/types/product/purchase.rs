use mongodb::bson::DateTime;
use stripe::ProductId;

use crate::database::{Collection, Id, UserId};

use super::PriceId;

pub type InvoiceId = Id<Invoice>;

/// The Invoice object represents an invoice made for a user
/// All disputes and refunds are done manually by an admin. An admin will determin how to effect the `Purchase` object on a case by case basis. (e.g. strip the user of the item or not)
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Invoice {
	#[serde(rename = "_id")]
	pub id: InvoiceId,
	/// The person who the invoice is for
	pub user_id: UserId,
	pub state: InvoiceState,
	pub purchases_id: Vec<PurchaseId>,
	// Additional data about the payment (e.g. amount, currency, stripe payment id, etc)
	// ...
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum InvoiceState {
	Draft,
	Finalized,
	Failed,
	Successful,
	Refunded {
		// Additional data about the refund
		// ...
	},
	Disputed {
		// Additional data about the dispute
		// ...
	},
}

impl Collection for Invoice {
	const COLLECTION_NAME: &'static str = "invoices";
}

pub type PurchaseId = Id<Purchase>;

/// A purchase is an item that a user owns
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Purchase {
	#[serde(rename = "_id")]
	pub id: PurchaseId,
	/// The person who receives the purchase
	pub user_id: UserId,
	/// A purchase can either be a subscription or a product
	/// Subscription is a product that requires renewal
	/// Product is a one-time purchase which does not require renewal
	pub data: PurchaseData,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum PurchaseData {
	Subscription {
		/// The id of the subscription at the provider (Stripe or Paypal)
		/// Might be None for old subscriptions
		subscription_provider_id: Option<String>,
		/// Status of the subscription
		recurring: Option<PurchaseRecurring>,
		/// Allow a grace period for the subscription
		allow_grace_period: bool,
		/// The different periods of time the subscription is active for or is currently active for
		periods: Vec<PurchaseSubscriptionPeriod>,
		/// Future periods that are scheduled to be created
		future_periods: Vec<PurchaseSubscriptionFuturePeriod>,
	},
	Product {
		/// The product that was purchased
		price_id: PriceId,
		/// The payment that was made
		invoice_id: InvoiceId,
	},
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PurchaseRecurring {
	/// The date the subscription was last renewed
	pub renewed_at: DateTime,
	/// Next renewal date
	pub next_renewal_at: DateTime,
	/// The payment for the subscription failed
	pub payment_failed: bool,
	/// The subscription was cancelled
	pub cancelled: bool,
}

/// Each payment represents a period of time the subscription is active for
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PurchaseSubscriptionPeriod {
	pub price_id: PriceId,
	/// This is the start time of the period, this is not the time the payment was made.
	/// because if the payment was made late due to whatever reason, and the subscription was in a grace period
	/// we wouls start this period from the end of the previous period to make sure no gap is created.
	pub start: DateTime,
	/// The time that this period ends. After this time the effect of the subscription is no longer active, unless there is a grace period.
	/// If the subscription is in a grace period, the subscription is still active until the end of the grace period.
	pub end: DateTime,
	/// The payment that created this period.
	pub created_by: PurchaseSubscriptionPeriodCreatedBy,
	/// If this period is enabled or not, typically this is not enabled until the invoice is paid.
	/// Or if the invoice is refunded.
	pub enabled: bool,
	// tier: SubscriptionTier,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PurchaseSubscriptionFuturePeriod {
	/// The time that this period ends. After this time the effect of the subscription is no longer active, unless there is a grace period.
	/// If the subscription is in a grace period, the subscription is still active until the end of the grace period.
	pub end: Option<DateTime>,
	/// The payment that created this period.
	pub created_by: PurchaseSubscriptionPeriodCreatedBy,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum PurchaseSubscriptionPeriodCreatedBy {
	/// The payment was made by the user
	Invoice { invoice_id: InvoiceId },
	/// The payment was made by a different user
	Gift { user_id: UserId, invoice_id: InvoiceId },
	/// This is a trial period that was created by the system
	Trial {},
}

impl Collection for Purchase {
	const COLLECTION_NAME: &'static str = "purchases";
}
