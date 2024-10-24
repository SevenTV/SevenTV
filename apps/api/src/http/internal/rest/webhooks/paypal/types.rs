use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use shared::database::product::invoice::InvoiceDisputeStatus;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Event {
	pub id: String,
	pub create_time: chrono::DateTime<chrono::Utc>,
	pub event_type: EventType,
	#[serde(flatten)]
	pub resource: Resource,
}

#[derive(Debug, Copy, Clone, serde::Deserialize)]
pub enum EventType {
	#[serde(rename = "PAYMENT.SALE.COMPLETED")]
	PaymentSaleCompleted,
	#[serde(rename = "PAYMENT.SALE.REFUNDED")]
	PaymentSaleRefunded,
	#[serde(rename = "PAYMENT.SALE.REVERSED")]
	PaymentSaleReversed,
	#[serde(rename = "CUSTOMER.DISPUTE.CREATED")]
	CustomerDisputeCreated,
	#[serde(rename = "CUSTOMER.DISPUTE.UPDATED")]
	CustomerDisputeUpdated,
	#[serde(rename = "CUSTOMER.DISPUTE.RESOLVED")]
	CustomerDisputeResolved,
	#[serde(rename = "BILLING.SUBSCRIPTION.EXPIRED")]
	BillingSubscriptionExpired,
	#[serde(rename = "BILLING.SUBSCRIPTION.CANCELLED")]
	BillingSubscriptionCancelled,
	#[serde(rename = "BILLING.SUBSCRIPTION.SUSPENDED")]
	BillingSubscriptionSuspended,
	#[serde(rename = "BILLING.SUBSCRIPTION.PAYMENT.FAILED")]
	BillingSubscriptionPaymentFailed,
}

impl EventType {
	pub const fn as_str(&self) -> &'static str {
		match self {
			EventType::PaymentSaleCompleted => "PAYMENT.SALE.COMPLETED",
			EventType::PaymentSaleRefunded => "PAYMENT.SALE.REFUNDED",
			EventType::PaymentSaleReversed => "PAYMENT.SALE.REVERSED",
			EventType::CustomerDisputeCreated => "CUSTOMER.DISPUTE.CREATED",
			EventType::CustomerDisputeUpdated => "CUSTOMER.DISPUTE.UPDATED",
			EventType::CustomerDisputeResolved => "CUSTOMER.DISPUTE.RESOLVED",
			EventType::BillingSubscriptionExpired => "BILLING.SUBSCRIPTION.EXPIRED",
			EventType::BillingSubscriptionCancelled => "BILLING.SUBSCRIPTION.CANCELLED",
			EventType::BillingSubscriptionSuspended => "BILLING.SUBSCRIPTION.SUSPENDED",
			EventType::BillingSubscriptionPaymentFailed => "BILLING.SUBSCRIPTION.PAYMENT.FAILED",
		}
	}
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "resource_type", content = "resource", rename_all = "snake_case")]
pub enum Resource {
	Sale(Sale),
	Refund(Refund),
	Dispute(Dispute),
	Subscription(Box<Subscription>),
}

/// https://developer.paypal.com/docs/api/payments/v1/#definition-sale
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Sale {
	pub id: String,
	// pub state: SaleState,
	pub amount: Amount,
	/// Only present if the sale is for a subscription
	pub billing_agreement_id: Option<String>,
	// pub create_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleState {
	Completed,
	PartiallyRefunded,
	Pending,
	Refunded,
	Denied,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Amount {
	/// The total amount of the sale as a decimal number.
	/// Negative on refunds.
	pub total: String,
	#[serde(deserialize_with = "stripe_currency")]
	pub currency: stripe::Currency,
}

fn stripe_currency<'de, D>(deserializer: D) -> Result<stripe::Currency, D::Error>
where
	D: Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?.to_lowercase();
	stripe::Currency::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Refund {
	pub sale_id: String,
}

/// https://developer.paypal.com/docs/api/customer-disputes/v1/#definition-dispute
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Dispute {
	pub dispute_id: String,
	pub status: DisputeStatus,
	pub disputed_transactions: Vec<DisputedTransaction>,
	// pub dispute_amount: Amount,
	// pub create_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DisputedTransaction {
	/// hopefully corresponds to a sale id Clueless
	pub seller_transaction_id: String,
}

#[derive(Debug, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeStatus {
	Open,
	WaitingForBuyerResponse,
	WaitingForSellerResponse,
	UnderReview,
	Resolved,
	Other,
}

impl From<DisputeStatus> for InvoiceDisputeStatus {
	fn from(value: DisputeStatus) -> Self {
		match value {
			DisputeStatus::Open => InvoiceDisputeStatus::UnderReview,
			DisputeStatus::WaitingForBuyerResponse => InvoiceDisputeStatus::NeedsResponse,
			DisputeStatus::WaitingForSellerResponse => InvoiceDisputeStatus::NeedsResponse,
			DisputeStatus::UnderReview => InvoiceDisputeStatus::UnderReview,
			DisputeStatus::Resolved => InvoiceDisputeStatus::Resolved,
			DisputeStatus::Other => InvoiceDisputeStatus::UnderReview,
		}
	}
}

/// https://developer.paypal.com/docs/api/subscriptions/v1/#definition-subscription
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Subscription {
	pub id: String,
	// pub status: SubscriptionStatus,
	// pub status_update_time: chrono::DateTime<chrono::Utc>,
	pub subscriber: Subscriber,
	pub billing_info: SubscriptionBillingInfo,
	pub plan_id: String,
	// pub create_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionStatus {
	ApprovalPending,
	Approved,
	Active,
	Suspended,
	Cancelled,
	Expired,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Subscriber {
	pub payer_id: String,
	pub email_address: Option<String>,
	pub name: Option<SubscriberName>,
	pub phone: Option<SubscriberPhone>,
	pub shipping_address: Option<SubscriberShippingAddress>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubscriberName {
	pub given_name: Option<String>,
	pub surname: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubscriberPhone {
	pub phone_number: Option<SubscriberPhoneNumber>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubscriberPhoneNumber {
	pub national_number: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubscriberShippingAddress {
	pub address_line_1: Option<String>,
	pub address_line_2: Option<String>,
	pub admin_area_1: Option<String>,
	pub admin_area_2: Option<String>,
	pub postal_code: Option<String>,
	pub country_code: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubscriptionBillingInfo {
	pub last_payment: Option<SubscriptionPayment>,
	// pub last_failed_payment: Option<SubscriptionPayment>,
	pub next_billing_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SubscriptionPayment {
	// pub amount: Amount,
	pub time: chrono::DateTime<chrono::Utc>,
}
