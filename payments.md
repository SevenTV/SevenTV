**OUTDATED**

# Stripe Payments

https://docs.stripe.com/api/checkout/sessions

https://docs.stripe.com/api/checkout/sessions/create#create_checkout_session-invoice_creation

As far as I can tell, by default invoices do not get generated for one-time payments.
Unless you set the `invoice_creation.enabled` to true in the checkout session.
Invoices are always generated for subscriptions.

## Events

https://docs.stripe.com/api/events/types

Once the user completes this checkout session we will get a webhook event: 

### Invoices

Invoices are for both subscriptions and one-time payments.

- `invoice.created`

When we receive this event we should create an `Invoice` object for the user. This object will contain the payment information and the items that were purchased. If the invoice is created for a subscription and that subscription is a `Gifted` period (which would mean this invoice is has a 100% discount) we should not create a `Invoice` object since this would imply the invoice is owned by another user and is not related to the user who received the gifted subscription.

- `invoice.paid`

Once we receive this event we would set the `Invoice` status to `Successful`. And would update the `Purchase` object to reflect that the user now owns the item.

Sometimes invoices are for subscriptions

- `invoice.payment_failed`

When we receive this event we would set the `Invoice` status to `Failed`. We would notify the user that their payment has failed and that they should update their payment method. We would also mark the `Purchase` object as not owned by the user.

### Subscription Only

- `customer.subscription.created`

When we receive this event we will check the `status` of the subscription. 

- `customer.subscription.deleted`

This event reflects that the subscription should be instantly deleted and the user should no longer have access to the subscription. This event is only emitted when the subscription has ended (e.g. at the end of a billing period or if they have been instantly cancelled and refunded).

- `customer.subscription.updated`

This is a very important event because we keep track of periods of the subscription. When we receive this event we should create a new `PurchaseSubscriptionPeriod`. We should check the metadata of the subscription to see if this is a gifted period or if its a trial or normal period. We should also check if the subscription's payment is past due or failed.


### Disputes

- `charge.dispute.created`

When a dispute is created we would mark the `Invoice` as disputed. We would create a `Ticket` to track the dispute.

- `charge.dispute.updated`

We would update the `Ticket` object with the new information about the dispute.

- `charge.dispute.closed`

We would close the `Ticket` object and update the `Invoice` object to reflect the outcome of the dispute. If the dispute was won by the user we would refund the payment. Otherwise we would mark the payment back to `Successful`. I am unsure if a `charged.refund` event is emitted when the dispute is won by the user. TODO: VERIFY

### Refunds

- `charge.refunded`

When a refund is created we would mark the `Invoice` as refunded and will create a `Ticket` for an admin to review what should happen to the `Purchase` object.

## Gifted Subscriptions

When a user gets gifted a subscription there are 2 states:

1. The user is already a subscriber

If the user is already a subscriber we would convert their subscription to a `SubscriptionSchedule` stripe object. This object allows us to specify phases of the subscription. We would set the end date of the current phase to the next billing cycle and then create a new phase after that with a coupon applied which renders the item free. This "gifted phase" will correspond to the amount of time that was gifted to the user. After this phases has concluded the subscription will revert back to the original plan. This is only the case if the original plan did not have an end date and was indefinite. There are cases where the original plan has an end date. For example: 

The user cancels their subscription at the end of the billing cycle and is then gifted 6 months of additional subscription. In this case we would create a new phase after their cancel date and then apply the coupon for those 6 months. After the 6 months the subscription would end.

The user is already on a gifted subscription and is then gifted another subscription. In this case we would create a new phase after the end of the current gifted phase and apply the coupon for the new gifted period. If the original plan had an end date we would not create a new phase at the end of the 2nd gifted period and the subscription would end. If the original plan did not have an end date we would create a new phase at the end of the 2nd gifted period and the subscription would revert back to the original plan.

2. The user is not a subscriber

If the user is not already a subscriber, we would create a new subscription that has a coupon applied and an end date of when the gifted period ends.

# PayPal Payments

For PayPal we will not support any new payments made through paypal and therefore will only allow existing subscriptions to continue to be paid through paypal. We will not support any new subscriptions through paypal.

Therefore we drastically reduce the complexity of the system by only supporting stripe payments.

## Events

https://developer.paypal.com/api/rest/webhooks/event-names

- `PAYMENT.SALE.COMPLETED`

This event is emitted when a payment is made on a subscription. We would create the `Invoice` object for the user who bought the item, and the `Purchase` object for the user who received the item.

- `PAYMENT.SALE.REFUNDED`

This event is emitted when a refund is made on a subscription. We would mark the `Invoice` as refunded and will create a `Ticket` for an admin to review what should happen to the `Purchase` object.

- `PAYMENT.SALE.REVERSED`

This event is emitted when a payment is reversed on a subscription. We would mark the `Invoice` as refunded and will create a `Ticket` for an admin to review what should happen to the `Purchase` object.

- `BILLING.SUBSCRIPTION.EXPIRED`

This event is emitted when a subscription has expired. This is equivalent to the `customer.subscription.deleted` event in stripe. We would mark the subscription as expired and the user should no longer have access to the subscription. I do not know if this event is emitted when a subscription is cancelled at the end of the cycle. TODO: VERIFY

- `BILLING.SUBSCRIPTION.CANCELLED`

This event is emitted at the point we cancel a subscription, its unclear how you would cancel a subscription at the end of the cycle. I think what happens is when you cancel a subscription, the subscription will continue and then issue an `BILLING.SUBSCRIPTION.EXPIRED` event when the subscription has ended. (I THINK NOT SURE, TODO: VERIFY)

- `BILLING.SUBSCRIPTION.PAYMENT.FAILED`

This event is emitted when a payment has failed for a subscription. It is equivalent to the `payment_intent.payment_failed` event in stripe. We would mark the subscription as payment failed and notify the user to update their payment method. I am not sure if we should allow users to update their payment method and instead force them to create a new subscription on stripe.

- `CUSTOMER.DISPUTE.CREATED`

This is equivalent to the `charge.dispute.created` event in stripe. We would mark the `Invoice` as disputed and will create a `Ticket` to track the dispute.

- `CUSTOMER.DISPUTE.RESOLVED`

This is equivalent to the `charge.dispute.closed` event in stripe. We would close the `Ticket` object and update the `Invoice` object to reflect the outcome of the dispute. If the dispute was won by the user we would refund the payment. Otherwise we would mark the payment back to `Successful`. I think if the dispute is won by the user paypal emits a `PAYMENT.SALE.REVERSED` event so I am unsure if we need to do something extra here. TODO: VERIFY

- `CUSTOMER.DISPUTE.UPDATED`

This is equivalent to the `charge.dispute.updated` event in stripe. We would update the `Ticket` object with the new information about the dispute.

# Rust Data Models

These datamodels would be designed to work with Stripe's API specifically. We will then map the PayPal API to these datamodels.

It is not immediately clear on how that mapping will look and we will have to investigate that further. 

TODO: Investigate & draft a spec on how to map PayPal API to Stripe API.
TODO: Finish the datamodels for the Stripe API.

```rs
// The Invoice object represents an invoice made for a user
// All disputes and refunds are done manually by an admin. An admin will determin how to effect the `Purchase` object on a case by case basis. (e.g. strip the user of the item or not)
struct Invoice {
  _id: Id<Invoice>,
  // The person who the invoice is for
  user_id: Id<User>,
  state: InvoiceState,
  purchases_id: Vec<Id<Purchase>>,
  // Additional data about the payment (e.g. amount, currency, stripe payment id, etc)
  // ... 
}

enum InvoiceState {
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
  }
}

// A purchase is an item that a user owns
struct Purchase {
  _id: Id<Purchase>,
  // The person who receives the purchase
  user_id: Id<User>,
  // A purchase can either be a subscription or a product
  // Subscription is a product that requires renewal
  // Product is a one-time purchase which does not require renewal
  data: PurchaseData,
}

enum PurchaseData {
  Subscription {
    // The plan that was subscribed to
    subscription_id: Id<Subscription>,
    // Status of the subscription
    recurring: Option<PurchaseRecurring>,
    // Allow a grace period for the subscription
    allow_grace_period: bool,
    // The different periods of time the subscription is active for or is currently active for
    periods: Vec<PurchaseSubscriptionPeriod>,
    // Future periods that are scheduled to be created
    future_periods: Vec<PurchaseSubscriptionFuturePeriod>,
  }
  Product {
    // The product that was purchased
    product_id: Id<Product>,
    // The payment that was made
    invoice_id: Id<Invoice>,
  }
}

struct PurchaseRecurring {
  // The date the subscription was last renewed
  renewed_at: DateTime,
  // Next renewal date
  next_renewal_at: DateTime,
  // The payment for the subscription failed
  payment_failed: bool,
  // The subscription was cancelled
  cancelled: bool,
}

// Each payment represents a period of time the subscription is active for
struct PurchaseSubscriptionPeriod {
  // This is the start time of the period, this is not the time the payment was made.
  // because if the payment was made late due to whatever reason, and the subscription was in a grace period
  // we wouls start this period from the end of the previous period to make sure no gap is created.
  start: DateTime,
  // The time that this period ends. After this time the effect of the subscription is no longer active, unless there is a grace period.
  // If the subscription is in a grace period, the subscription is still active until the end of the grace period.
  end: DateTime,
  // The payment that created this period.
  created_by: PurchaseSubscriptionPeriodCreatedBy,
  // If this period is enabled or not, typically this is not enabled until the invoice is paid.
  // Or if the invoice is refunded.
  enabled: bool,
  // tier: SubscriptionTier,
}

struct PurchaseSubscriptionFuturePeriod {
  // The time that this period ends. After this time the effect of the subscription is no longer active, unless there is a grace period.
  // If the subscription is in a grace period, the subscription is still active until the end of the grace period.
  end: Option<DateTime>,
  // The payment that created this period.
  created_by: PurchaseSubscriptionPeriodCreatedBy,
}

enum PurchaseSubscriptionPeriodCreatedBy {
    // The payment was made by the user
    Invoice {
        invoice_id: Id<Invoice>,
    },
    // The payment was made by a different user
    Gift {
        user_id: Id<User>,
        invoice_id: Id<Invoice>,
    },
    // This is a trial period that was created by the system
    Trial {},
}

```