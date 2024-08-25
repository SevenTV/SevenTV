# Types

## Products

```rs
/// e.g. a paint bundle
struct Product {
    id: stripe::PriceId,
    name: String,
    description: String,
    default_currency: stripe::Currency,
    currency_prices: HashMap<stripe::Currency, u64>,
}

/// There are only two kinds of subscriptions: monthly and yearly.
struct SubscriptionProduct {
    id: stripe::PriceId,
    name: String,
    description: String,
    default_currency: stripe::Currency,
    currency_prices: HashMap<stripe::Currency, u64>,
    kind: SubscriptionKind,
    benefits: Vec<SubscriptionBenefit>,
}

/// An entitlement edge between the user sub and `entitlement` which will be inserted after the duration `after` has passed.
struct SubscriptionBenefit {
    entitlement: EntitlementEdgeKind,
    after: Duration,
}

enum SubscriptionKind {
    Monthly,
    Yearly,
}
```

## Invoice

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
struct SubscriptionPeriod {
    id: SubscriptionPeriodId,
    subscription_id: stripe::SubscriptionId,
    user_id: UserId,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    is_trial: bool,
    created_by: SubscriptionCreatedBy,
    product_id: stripe::PriceId,
}

enum SubscriptionCreatedBy {
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
