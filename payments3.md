# Types

## Products

```rs
stripe_type!(ProductId, stripe::PriceId);
stripe_type!(SubscriptionId, stripe::SubscriptionId);
stripe_type!(InvoiceId, stripe::InvoiceId);
stripe_type!(InvoiceLineItemId, stripe::InvoiceLineItemId);
stripe_type!(CustomerId, stripe::CustomerId);
stripe_type!(PaymentIntentId, stripe::PaymentIntentId);

/// A non-recurring product, e.g. a paint bundle
pub struct Product {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: ProductId,
	pub name: String,
	pub description: Option<String>,
	pub extends_subscription: Option<SubscriptionProductId>,
	pub default_currency: stripe::Currency,
	pub currency_prices: HashMap<stripe::Currency, i32>,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

enum SubscriptionProductProviderId {
	Stripe(ProductId),
	Paypal(String)
}

pub struct SubscriptionProductVariant {
	pub id: SubscriptionProductProviderId,
	pub kind: SubscriptionProductKind,
	pub currency_prices: HashMap<stripe::Currency, i32>,
}

/// There are only two kinds of subscriptions: monthly and yearly.
pub struct SubscriptionProduct {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionProductId,
	pub variants: Vec<SubscriptionProductVariant>
	pub name: String,
	pub description: Option<String>,
	pub default_currency: stripe::Currency,
	pub benefits: Vec<SubscriptionBenefit>,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// An entitlement edge between the user sub and `entitlement` which will be
/// inserted as soon as the condition is met.
/// The `SubscriptionBenefitId` can have entitlements attached via the entitlement graph.
/// If the user qualifies for the entitlement benifit then we create an edge between `Subscription` and `SubscriptionBenefit` on the entitlement graph.
pub struct SubscriptionBenefit {
	pub id: SubscriptionBenefitId,
	pub condition: SubscriptionBenefitCondition,
}

pub enum SubscriptionBenefitCondition {
	Duration(DurationUnit),
	TimePeriod(TimePeriod),
}

pub enum SubscriptionProductKind {
	Monthly = 0,
	Yearly = 1,
}
```

## Invoice

Invoices are just for showing them to the user.
They are not technically necessary.

```rs
/// Only for showing to the user.
/// Technically not necessary.
pub struct Invoice {
	/// This ID will be the stripe ID for the invoice
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: InvoiceId,
	/// These items will be the stripe line items for the invoice
	pub items: Vec<ProductId>,
	/// customer id from stripe
	pub customer_id: CustomerId,
	/// User who the invoice is for
	pub user_id: UserId,
	/// If this invoice was paid via a legacy payment
	pub paypal_payment_id: Option<String>,
	/// Status of the invoice
	pub status: InvoiceStatus,
	/// If a payment failed
	pub failed: bool,
	/// If the invoice was refunded
	pub refunded: bool,
	/// If the invoice was disputed
	pub disputed: Option<InvoiceDisputeStatus>,
	#[serde(with = "crate::database::serde")]
	/// Created at
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	/// Updated at
	pub updated_at: chrono::DateTime<chrono::Utc>,
	/// Search updated at
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub enum InvoiceDisputeStatus {
	Lost = 0,
	NeedsResponse = 1,
	UnderReview = 2,
	WarningClosed = 3,
	WarningNeedsResponse = 4,
	WarningUnderReview = 5,
	Won = 6,
	/// only applies to paypal disputes, either won or lost, we don't know
	Resolved = 7,
}
```

## Subscription

```rs
pub struct SubscriptionId {
	pub user_id: UserId,
	pub product_id: SubscriptionProductId,
}

/// All subscriptions that ever existed, not only active ones
/// This is only used to save data about a subscription that could also be
/// retrieved from Stripe or PayPal It is used to avoid sending requests to
/// Stripe or PayPal every time someone queries data about a subscription
pub struct Subscription {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionId,
	pub state: SubscriptionState,
	// This value will be populated by the refresh function, it is computed from the `SubscriptionPeriod`s for this `Subscription`
	pub trial_end: Option<chrono::DateTime<chrono::Utc>>,
	// This value will be populated by the refresh function, it is computed from the `SubscriptionPeriod`s for this `Subscription`
	pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
	#[serde(with = "crate::database::serde")]
	pub created_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Subscription {
	/// does not need to be on the object can be a standalone that takes the object id idk up to u
	async fn refresh(&self, ...) -> Result<(), ...> {
		/// This function will refresh the subscription object and entitlement edges if the conditions are met for the benifits.
	}
}

enum SubscriptionState {
	CancelAtEnd,
	Active,
	Ended,
}

enum SubscriptionProviderId {
	Stripe(stripe::SubscriptionId),
	Paypal(String),
}

/// Current or past periods of a subscription (not future)
pub struct SubscriptionPeriod {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SubscriptionPeriodId,
	pub subscription_id: SubscriptionId,
	pub provider_id: SubscriptionProviderId,
	#[serde(with = "crate::database::serde")]
	pub start: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub end: chrono::DateTime<chrono::Utc>,
	pub is_trial: bool,
	pub created_by: SubscriptionPeriodCreatedBy,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub enum SubscriptionPeriodCreatedBy {
	RedeemCode { redeem_code_id: RedeemCodeId },
	Invoice { invoice_id: InvoiceId },
	Gift { gifter: UserId, payment: PaymentIntentId },
	System { reason: Option<String> },
}
```

## Codes

```rs
pub struct RedeemCode {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub remaining_uses: i32,
	pub active_period: TimePeriod,
	pub effects: Vec<CodeEffect>,
	pub created_by: UserId,
	pub special_event_id: Option<SpecialEventId>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub enum CodeEffect {
	Entitlement { edge: EntitlementEdgeKind, extend_subscription: Option<SubscriptionProductId> },
	/// This isnt an `Entitlement` we should just create a Subscription object for this user and then give them a period for the trial days.
	/// If its only for first_time subs then dont give if they already have subbed before, and only give if they are not actively subbed.
	/// Never create a new period if they are actively subbed.
	SubscriptionProduct { id: SubscriptionProductId, trial_days: u32, first_time_only: bool },
}

// Display purposes only.
pub struct SpecialEvent {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: SpecialEventId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

# Webhooks

## Stripe Webhooks

### `customer.created`

Update user with stripe customer id.

### `checkout.session.completed`

If it is a setup session, update the user's default payment methods.

If the session is related to a redeem code, grant access to the entitlements linked to the redeem code.

If the session is related to a gift subscription payment, add one subscription period for the recipient.

### `checkout.session.expired`

If the session is related to a redeem code, increase the remaining uses of the redeem code again because it was left unused.

### `invoice.created`

Create the invoice object and finalize it.

### `invoice.updated`, `invoice.finalized`, `invoice.payment_succeeded`

Update the invoice object.

### `invoice.paid`

Update the invoice object.
If invoice is for a subscription, add a new subscription period to that subscription.
If the subscription is still trial, make the period a trial period.
We should probably add a grace period of a few days here.

### `invoice.deleted`

Only sent for draft invoices.
Delete the invoice object.

### `invoice.payment_failed`

Mark the associated invoice as failed.
Show the user an error message.
Collect new payment information and update the subscriptions default payment method.

### `customer.subscription.deleted`

Set the subscription period end to `ended_at`. (which means ending the current period right away)

### `customer.subscription.updated`

End the current subscription period right away when the subscription products got removed from the subscription.
Otherwise, update the current subscription period to include all updated subscription products.

### `charge.refunded`

Mark associated invoice as refunded.

### `charge.dispute.created`

Mark the associated invoice as disputed.

### `charge.dispute.updated`

Update the associated invoice.

### `charge.dispute.closed`

Update the associated invoice with the outcome of the dispute.

## PayPal Webhooks

https://developer.paypal.com/api/rest/webhooks/event-names/#link-subscriptions

### `PAYMENT.SALE.COMPLETED`

https://stackoverflow.com/a/61530219/10772729
When a payment is made on a subscription.

Create a new Stripe customer for the paypal customer.
Create the invoice in Stripe.
Create the invoice object.
Finalize the invoice.
Void the invoice which doesn't actually charge anything.
Fetch the subscription from Paypal.
Add a new subscription period to the subscription which starts `billing_info.last_payment.time` and ends `billing_info.next_billing_time`.
We should probably add a grace period of a few days here.

### `BILLING.SUBSCRIPTION.CANCELLED`, `BILLING.SUBSCRIPTION.SUSPENDED`

End the current period right away.

### (`BILLING.SUBSCRIPTION.EXPIRED`)

According to gpt, this means that the subscription was cancelled but should be ended at the end of the current period.
Which means that we don't have to do anything when we receive this event.

### (`BILLING.SUBSCRIPTION.PAYMENT.FAILED`)

We only get the subscription id here.
There is no invoice for this payment because it only gets created when the payment succeeds.
Show the user an error message.
Collect new payment information and update the subscriptions default payment method.

### `PAYMENT.SALE.REFUNDED`, `PAYMENT.SALE.REVERSED`

Mark associated invoice as refunded.

### `CUSTOMER.DISPUTE.CREATED`, `CUSTOMER.DISPUTE.UPDATED`, `CUSTOMER.DISPUTE.RESOLVED`

Mark the associated invoice as disputed.
Assuming that the `seller_transaction_id` is the sale id.

# Egvault endpoints

## `POST /subcriptions`

Search for existing stripe customer.
If not found, create new stripe customer.
Create new stripe subscription.

## `GET /subscriptions/:target`

Aggregate subscription perdiods.

## `DELETE /subscriptions/:target`

End at period end.

## `POST /subscription/:target/reactivate`

Don't end at period end.

## `PATCH /subscription/:target/payment-method`

Update the user's default payment method.

## `GET /products`

Return list of subscription products.

## `POST /redeem`

Insert entitlement edges.
