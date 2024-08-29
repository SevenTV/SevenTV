**OUTDATED**

# Stripe Events are not guaranteed to be delivered in order.

## Events

### Stripe

#### Invoice

- `invoice.created`

- `invoice.updated`

- `invoice.deleted`

- `invoice.paid`

- `invoice.payment_failed`

- `invoice.voided`

- `invoice.marked_uncollectible`

#### Subscription

- `customer.subscription.created`

- `customer.subscription.deleted`

- `customer.subscription.updated`

#### Charge

- `charge.refunded`

- `charge.dispute.created`

- `charge.dispute.updated`

- `charge.dispute.closed`

### PayPal

#### Invoice

- `PAYMENT.SALE.COMPLETED`

    Equivalent to `invoice.paid`

- `PAYMENT.SALE.REFUNDED`

    Equivalent to `charge.refunded`

- `PAYMENT.SALE.REVERSED`

    Equivalent to `charge.refunded`

### Charge

- `CUSTOMER.DISPUTE.CREATED`
    
    Equivalent to `charge.dispute.created`

- `CUSTOMER.DISPUTE.UPDATED`
    
    Equivalent to `charge.dispute.updated`

- `CUSTOMER.DISPUTE.RESOLVED`
    
    Equivalent to `charge.dispute.closed`

### Subscription

- `BILLING.SUBSCRIPTION.EXPIRED`
    
    Equivalent to `customer.subscription.deleted`

- `BILLING.SUBSCRIPTION.CANCELLED`
    
    Equivalent to `customer.subscription.deleted`

- `BILLING.SUBSCRIPTION.SUSPENDED`
    
    Equivalent to `customer.subscription.deleted`

- `BILLING.SUBSCRIPTION.PAYMENT.FAILED`
    
    Equivalent to `invoice.payment_failed` & `customer.subscription.updated` where status is `overdue`

## Rust Types

```rust

// An invoice that is generated for a purchase
struct Invoice {
    // This ID will be the stripe ID for the invoice
    _id: String,
    // These items will be the stripe line items for the invoice
    items: Vec<InvoiceItem>,
    // customer id from stripe
    customer_id: String,
    // User who the invoice is for
    user_id: Id<User>,
    // If this invoice was paid via a legacy payment
    paypal_payment_id: Vec<String>,
    // If the invoice was deleted
    status: InvoiceStatus,
}

enum InvoiceStatus {
    Pending,
    Failed,
    Deleted,
    Paid,
    Voided,
    Uncollectible,
}

struct InvoiceItem {
    // This will be a line item id from stripe
    id: String,
    // This is a stripe id for the product
    product: ProductRef,
    // User who the item is for
    user_id: Id<User>,
    // Other information about the item like quantity, price, subscription plan, etc.
}

// An item that can be purchased
struct Product {
    // This ID will be the stripe ID for the product
    _id: String,
    kind: ProductKind,
    // there will be other fields here like name, description, price, etc.
    // those fields will be shown in the UI but are not relevant to the core logic
    // We should also make those fields sync from Stripe.
    prices: Vec<ProductPrice>,
}

struct ProductPrice {
    id: String,
    // some other fields like currency, amount, etc.
}

// The kind of product
enum ProductKind {
    Subscription,
    OneTimePurchase,
}

struct InvoiceRef {
    // The invoice id
    id: String,
    // Optionally the item this reference refers to otherwise it is the whole invoice
    item_id: String,
}

struct ProductRef {
    // The invoice id
    id: String,
    // Optionally the item this reference refers to otherwise it is the whole invoice
    price_id: String,
}

// A purchase of a `Product`
// `Purchase` are always for products of kind `OneTimePurchase`
// Unlike the `Subscription`, a user can have multiple purchases of the same product.
struct Purchase {
    // This is a unique id for our system
    _id: Id<Purchase>,
    // The stripe id for the purchase (corrisponds to the `Product.id` field)
    product_id: String,
    // Our internal id for the user who received the purchase
    user_id: Id<User>,
    // The invoice that created this purchase
    invoice: InvoiceRef,
    // If this item has been refunded
    refuned: bool,
}

// A subscription to a `Product`
// `Subscription` are always for products of kind `Subscription`
struct Subscription {
    // The stripe id for the subscription
    _id: String,
    // Product id for the subscription
    product_id: String,
    // The user who received the subscription
    user_id: Id<User>,
    // Start time of the subscription
    start: DateTime<Utc>,
    // The periods of this subscription (these cannot overlap and are in order)
    periods: Vec<SubscriptionPeriod>,
    // Future periods that are scheduled to start 
    scheduled_periods: Vec<SubscriptionScheduledPeriod>,
    // The status of this subscription
    standing: Option<SubscriptionStanding>,
    // Legacy PayPal subscription
    paypal_subscription: Option<PayPalSubscription>,
    // If the subscription is active or not
    state: SubscriptionState,
}

struct PayPalSubscription {
    id: String,
    state: Option<PayPalSubscriptionState>,
}

enum PayPalSubscriptionState {
    Payment(String),
    Invoice(String),
}

enum SubscriptionState {
    // The subscription is active
    Active,
    // The subscription is paused
    Ended,
    // The subscription is in an invalid state. (however it is still active)
    Invalid,
}

enum SubscriptionStanding {
    // Cancelled by the user (ends at the end of the current period)
    Cancelled,
    // The subscription payment is overdue (active but payment has failed)
    Overdue,
}

struct SubscriptionScheduledPeriod {
    // The end time of the period will always be in the future
    end: DateTime<Utc>,
    // How this period was created
    special_kind: Option<SubscriptionPeriodSpecialKind>,
    // The price id for the period
    product_price_id: String,
}

// Subscription Periods only have a single end time because they are always contiguous
struct SubscriptionPeriod {
    // The end time of the period (may be in the future if the period is ongoing)
    end: DateTime<Utc>,
    // How this period was created (if none then it is a normal period)
    special_kind: Option<SubscriptionPeriodSpecialKind>,
    // The invoice that created this period
    invoice_id: String,
    // If this period is enabled.
    enabled: bool,
    // Price id for the period
    product_price_id: String,
}

enum SubscriptionPeriodSpecialKind {
    // A trial period
    Trial {
        // The reason for the trial
        reason: Option<String>,
    },
    // A gifted period
    Gift {
        // The user who gifted the period
        gifter_id: Id<User>,
        // The invoice that created the gift
        invoice: InvoiceRef,
    },
    // A period created by the system
    System {
        reason: String,
    },
}

```
