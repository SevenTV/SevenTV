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
