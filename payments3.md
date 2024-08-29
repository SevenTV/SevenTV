**MIGHT BE OUTDATED IN SOME PLACES**

# Types

## Products

```rs
/// A non-recurring product, e.g. a paint bundle
struct Product {
    id: stripe::PriceId,
    name: String,
    description: String,
    default_currency: stripe::Currency,
    currency_prices: HashMap<stripe::Currency, i64>,
}

/// There are only two kinds of subscriptions: monthly and yearly.
struct SubscriptionProduct {
    id: stripe::PriceId,
    name: String,
    description: String,
    default_currency: stripe::Currency,
    currency_prices: HashMap<stripe::Currency, i64>,
    kind: SubscriptionKind,
    benefits: Vec<SubscriptionBenefit>,
}

/// An entitlement edge between the user sub and `entitlement` which will be inserted after the duration `after` has passed.
struct SubscriptionBenefit {
    entitlement: EntitlementEdgeKind,
    condition: SubscriptionBenefitCondition,
}

enum SubscriptionBenefitCondition {
    DurationDays(i32),
    DurationMonths(i32),
    TimePeriod {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
}

enum SubscriptionKind {
    Monthly,
    Yearly,
}
```

## Invoice

Invoices are just for showing them to the user.
They are not technically necessary.

```rs
struct Invoice {
    id: stripe::InvoiceId,
    items: Vec<InvoiceItem>,
    customer_id: stripe::CustomerId,
    user_id: UserId,
    paypal_payment_ids: Vec<String>,
    status: InvoiceStatus,
    note: Option<String>,
}

struct InvoiceItem {
    product_id: stripe::PriceId,
    // ...
}

enum InvoiceStatus {
    // ...
}
```

## Subscription

```rs
/// Current or past periods of a subscription (not future)
struct SubscriptionPeriod {
    id: SubscriptionPeriodId,
    subscription_id: stripe::SubscriptionId,
    product_id: stripe::PriceId,
    user_id: UserId,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    is_trial: bool,
    created_by: PeriodCreatedBy,
}

enum PeriodCreatedBy {
    RedeemCode {
        redeem_code_id: RedeemCodeId,
    },
    GiftCode {
        gift_code_id: GiftCodeId,
    },
    Invoice {
        invoice_id: stripe::InvoiceId,
        invoice_item_id: stripe::InvoiceLineItemId,
    },
    System {
        reason: Option<String>,
    },
}
```

# Webhooks

## Stripe Webhooks

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

TODO

## `GET /products`

Return list of subscription products.

## `POST /redeem`

Insert entitlement edges.
