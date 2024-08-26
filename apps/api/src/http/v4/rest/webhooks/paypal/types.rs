#[derive(Debug, Clone, serde::Deserialize)]
pub struct Event {
	pub id: String,
	pub create_time: chrono::DateTime<chrono::Utc>,
	pub event_type: EventType,
	#[serde(flatten)]
	pub ressource: Resource,
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

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "resource_type", content = "resource", rename_all = "snake_case")]
pub enum Resource {
	Sale(Sale),
    Dispute(Dispute),
    Subscription(Subscription),
}

/// https://developer.paypal.com/docs/api/payments/v1/#definition-sale
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Sale {
	pub id: String,
    pub state: SaleState,
    pub amount: Amount,
	pub create_time: chrono::DateTime<chrono::Utc>,
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
    pub currency: stripe::Currency,
}

/// https://developer.paypal.com/docs/api/customer-disputes/v1/#definition-dispute
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Dispute {
    pub dispute_id: String,
    pub status: DisputeStatus,
    pub dispute_life_cycle_stage: DisputeLifeCycleStage,
    pub dispute_channel: DisputeChannel,
    pub reason: DisputeReason,
    pub dispute_amount: Amount,
    pub create_time: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeLifeCycleStage {
    Inquiry,
    Chargeback,
    PreArbitration,
    Arbitration,
}

#[derive(Debug, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeChannel {
    Internal,
    External,
    Alert,
}

#[derive(Debug, Copy, Clone, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisputeReason {
    MerchandiseOrServiceNotReceived,
    MerchandiseOrServiceNotAsDescribed,
    Unauthorised,
    CreditNotProcessed,
    DuplicateTransaction,
    IncorrectAmount,
    PaymentByOtherMeans,
    CanceledRecurringBilling,
    ProblemWithRemittance,
    Other,
}

/// https://developer.paypal.com/docs/api/subscriptions/v1/#definition-subscription
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Subscription {
    pub id: String,
    pub status: SubscriptionStatus,
    pub status_update_time: chrono::DateTime<chrono::Utc>,
    pub plan_id: String,
    pub create_time: chrono::DateTime<chrono::Utc>,
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
