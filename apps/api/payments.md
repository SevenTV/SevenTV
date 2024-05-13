# How payments will work

## Redeem codes

- Charge gifter with one-time payment

### Declined gift

- Refund gifter

### Accepted gift / redeemed code

#### Non-recurring price

- Create one-time purchase with 100% discount

#### Recurring price

- Convert current sub to subscription schedule if it isn't already
- Append a new phase to the schedule with the gifted price and a 100% discount
- Append a new phase that's the same as the previous one (if there was one)

## One time purchase

- Stripe PaymentIntent

## Subscription

- Create stripe subscription
