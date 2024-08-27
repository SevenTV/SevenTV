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

## Webhooks

### `invoice.created`

Create the invoice object and finalize it.

### `invoice.updated`, `invoice.finalized`, `invoice.payment_succeeded`

Update the invoice object.

### `invoice.paid`

Update the invoice object.
If invoice is for a subscription, add a new subscription period to that subscription.
If the subscription is still trial, make the period a trial period.

### `invoice.deleted`

Only sent for draft invoices.
Delete the invoice object.

### `invoice.payment_failed`

Show the user an error message.
Collect new payment information and update the subscriptions default payment method.

### `customer.subscription.deleted`

Set the subscription period end to `ended_at`.

### `customer.subscription.updated`

End the current subscription period right away when the subscription products got removed from the subscription.
Otherwise, update the current subscription period to include all updated subscription products.

### `charge.refunded`

Mark associated invoice as refunded.

### `charge.dispute.created`

Mark the associated invoice as disputed.

### `charge.dispute.closed`

Update the associated invoice with the outcome of the dispute.
