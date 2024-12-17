# Database Schema Overview

## Overview

We use a few different databases:

- `mongo` - The primary source of truth for all data.
- `clickhouse` - used for statistics and generating the trending emote leaderboard.
- `redis` - used for rate limiting
- `typesense` - used for search
- `nats` - used for inter-service communication as a message broker.

## Backups

We have a cronjob (kubernetes cronjob) that runs daily and creates a backup of the following databases:

- `mongo`
- `clickhouse`
- `redis`

These backups are stored indefinately in the a `backups` bucket on the Hetzner cloud account. (not wasabi)

## Mongo

Most of the collections have a `updated_at` and a `search_updated_at` column. These columns are used to automatically reindex the search index when a row (document) is updated.

Mongo requires that every collection have an `_id` column, and most times we use a `ULID` (mongo `UUID`) value for this column. There are a few tables that use a different `_id` structure, each has an explanation below.

### Badge

The collection name is: `badges`
ID Column Type: `ULID`

Description: We have a few different badges which are given to users for various reasons controlled via the `entitlement_edge` collection. This table contains all of the badges and where to find their images.

### CronJob

The collection name is: `cron_jobs`
ID Column Type: `i32` - We only have a few cron jobs all inserted during a migration and we want to keep track of them and the ids must be stable. See the `CronJobId` enum for the ids.

Description: Periodically we run a few different jobs, `EmoteScoresUpdate` is run every day and updates the trending emote leaderboard. `SubscriptionRefresh` is run every day and updates the subscription status for all users.

### EmoteModerationRequest

The collection name is: `emote_moderation_requests`
ID Column Type: `ULID`

Description: This table contains all of the moderation requests for emotes. Whenever a new emote is uploaded we create a moderation request for it, so that the moderators can review it and approve it or deny it.

### Emote

The collection name is: `emotes`
ID Column Type: `ULID`

Description: This table contains all of the emotes that are uploaded to the platform. Emotes which are deleted have a flag `deleted` set to true. Emotes can be merged into other emotes, this is tracked via the `merged` column. 

We also have a `scores` column which contains the trending scores for the emote. These are updated via the `EmoteScoresUpdate` cron job.

Future work:

Currently we do not flatten these emotes after some amount of time so merging can always be reverted. Merging negatively effects the database performance so eventually a mechnism to flatten these emotes is needed if there are a lof of merges.

### EmoteSet

The collection name is: `emote_sets`
ID Column Type: `ULID`

Description: This table contains all of the emote sets that are created by users. Emote sets are used to group emotes together for various reasons, such as a user's personal emote set or a global emote set or a channel emote set.

There are 4 types of emote sets:

- `Normal` - A normal emote set, can be applied to a channel
- `Personal` - A personal emote set, one per user (allows those emotes to be used by the user in chat)
- `Global` - A global emote set (enables emotes to be used by all users in chat)
- `Special` - A special emote set, like the personal emote set but multiple users can use it (if they have access to it via entitlements)

The `emotes_changed_since_reindex` column is used to track if the emotes in the set have changed since the last time the search index was reindexed. This is used to speed up the reindexing process.

Future work:

The design for emote set origins exists but is not fully implemented. The idea is to allow emote sets to depend on each other and then update them automatically.

### EntitlementEdge


The collection name is: `entitlement_edges`
ID Column Type: `{To, From, ManagedBy}`

Description: This table contains all of the entitlements that are given to users. Entitlements are used to grant users access to emote sets, badges, and other resources.

This table is a graph structure. There are a few different edge types:

- `User` - A user
- `Role` - A role
- `Badge` - A badge
- `Paint` - A paint
- `EmoteSet` - An emote set
- `Product` - A product
- `SubscriptionBenefit` - A subscription benefit
- `Subscription` - A subscription
- `SpecialEvent` - A special event
- `GlobalDefaultEntitlementGroup` - A global default entitlement group

Some of these edges can be bi-directional, others are only one way.

The following edges are bi-directional:

- `Role`
- `SubscriptionBenefit`
- `Subscription`
- `SpecialEvent`
- `Product`

The following edges are only one way:

- `User` - Can only have children
- `Badge` - Can only have parents
- `Paint` - Can only have parents
- `EmoteSet` - Can only have parents
- `GlobalDefaultEntitlementGroup` - Can only have children

Typically the `From` column is the "owner" of the edge and is responsible for managing the state of the edge (exists or not). However sometimes we want to create edges for items external to the item itself, the `ManagedBy` column is used to track this.

### Product

The collection name is: `products`
ID Column Type: `ULID`

Description: This table contains all of the products that are available for purchase. These are non-recurring products, e.g. a paint bundle.

Entitlements can be attached to products, this is done via the `entitlement_edges` collection.

Future work:

Currently we do not have any products that are non-recurring.

### SubscriptionProduct

The collection name is: `subscription_products`
ID Column Type: `ULID`

Description: This table contains all of the subscription products that are available for purchase. These are recurring products, e.g. a monthly or yearly subscription.

Subscriptions can have different variants, e.g. a monthly and yearly variant.

Subscriptions can have different currencies per variant, e.g a monthly variant can be $5.99 in USD and $7.99 in CAD.

Subscriptions have benefits which are used to determine what rewards are given to users who subscribe. These benefits have calculations. Currently we only have 2 possible calculations:

- `Duration` - A duration benefit, e.g. after 1 month of subscription
    - Used for doing the badge rewards (1 month badge, 3 month badge, etc)

- `TimePeriod` - A time period benefit, e.g. during the month of January
    - Used for doing the monthly paint bundles (January paint bundle, February paint bundle, etc)

Future work:

We currently only have a single subscription product & currencies although fully implemented we do not have any subscriptions which use them.

### RedeemCode

The collection name is: `redeem_codes`
ID Column Type: `ULID`

Description: This table contains all of the redeem codes that can be redeemed in order to get products or subscriptions.

Redeem codes can either be `DirectEntitlement` or `SpecialEvent`. 

- `DirectEntitlement` - A redeem code that gives entitlements directly to the user or attached to their subscription.
- `SpecialEvent` - A redeem code that is part of a special event, where we will create a Edge between the user (or their subscription) and the special event. Entitlements will then be attached to the special event.

Redeem codes can also have a `SubscriptionEffect` which is used to determine what happens if the user requires a subscription to use the effects of the redeem code. 

Redeem codes can have a limited number of uses and an active period.

### Invoice

The collection name is: `invoices`
ID Column Type: `StripeInvoiceId` (string)

Description: This table contains all of the invoices that are created for users when anything happens that requires a payment.

Generated using stripe webhook data.

### SpecialEvent

The collection name is: `special_events`
ID Column Type: `ULID`

Description: This table contains all of the special events, special events are like "groups of entitlements" that are created for a specific event, e.g. Nymn's New Year Show. 

Entitlements can be attached to special events, this is done via the `entitlement_edges` collection.

They are used to show that a user has access to this group of entitlements because they have access to the special event on the website, also allows to change the entitlements given to users after they already received them.

### Subscription

The collection name is: `subscriptions`
ID Column Type: `{UserId, SubscriptionProductId}`

Description: This table is essentially a cache of the current state of the user's subscription for a given subscription product.

It could probably be removed and replaced with a query that gets the current subscription from the `subscription_periods` collection.


### SubscriptionPeriod

The collection name is: `subscription_periods`
ID Column Type: `UUID`

Description: This table contains all of the periods of a subscription for a given subscription product.

This table is used to keep track of all the periods that a user has been subscribed to a given subscription product & how they were created.

This data is generated via Stripe webhook data.

### GlobalConfig

The collection name is: `global_config`
ID Column Type: `null`

Description: This table contains all of the global configuration for the platform.

This is used to store the global configuration for the platform, such as the alerts, emote set, trending emote count, and country currency overrides. There is only a single row in this table.

### Role

The collection name is: `roles`
ID Column Type: `ULID`

Description: This table contains all of the roles that are available for users to use.

Roles are used to grant users permissions to do things. Roles can also have entitlements attached to them, this is done via the `entitlement_edges` collection.

The `hoist` flag shows if the role should be shown separately in the role list.

The `applied_rank` column is used to check if we need to reindex the search index for the role.

### User

The collection name is: `users`
ID Column Type: `ULID`

Description: This table contains all of the users that are registered on the platform.

Users can have a `merged_into_id` which is used to track if the user was merged into another user. This is used to clean up users that were merged into another user.

Users can have a `stripe_customer_id` which is used to track the user's Stripe customer ID. This is used to manage the user's subscription.

Users can have a `cached` column which is a cache of their entitlements computed using the `entitlement_edges` collection. This is also used to find all users with a given entitlement.

The `has_bans` flag is used as an optimization to skip checking if the user has any bans. Only users with this flag set to true will be checked when checking if a user is currently banned.

Future work:

- We currently do not clean up users that are merged into another user.
- We currently do not save settings for users into the `settings` column.
- We currently do not have any users that have a `two_fa` column.

### UserBan

The collection name is: `user_bans`
ID Column Type: `ULID`

Description: This table contains all of the bans that are placed on users.

Bans can have a `removed` column which is used to track if the ban has been removed.

### UserEditor

The collection name is: `user_editors`
ID Column Type: `{UserId, UserId}`

Description: This table contains all of the editors (other users) for a given user.

Editors are used to allow users to edit each other's emotes.

### UserProfilePicture

The collection name is: `user_profile_pictures`
ID Column Type: `ULID`

Description: This table contains all of the profile pictures that are attached to users.

A profile picture can be enabled via the User `style` column.

Future work:

- We don't render past profile pictures they are only saved.

### UserSession

The collection name is: `user_sessions`
ID Column Type: `ULID`

Description: This table contains all of the sessions that are created for users.

Sessions are created when a user logs in and are deleted when a user logs out. They also automatically expire after a certain amount of time.

Sessions can have extensions which can be used to grant additional permissions to the session. (currently this is used as a poor-mans-oauth2, only used by 1 person which is the NNYS bot)

Future work:

- Add a fully implemented OAuth2 server

### Paint

The collection name is: `paints`
ID Column Type: `ULID`

Description: This table contains all of the paints that are available for users.

Paints are used to change the color of the user's emotes.

### StoredEvent

The collection name is: `stored_events`
ID Column Type: `ULID`

Description: This table contains all of the stored events that are created. (Audit Logs)

Stored events are used to track events that have happened on the platform.

Future work:

- We currently don't track every event that happens on the platform.

### StripeError

The collection name is: `stripe_errors`
ID Column Type: `ULID`

Description: This table contains all of the errors that are created when interacting with Stripe.

These errors are used to track errors that happen when interacting with Stripe, such as a webhook failing to process. 
We manually check from time to time to try debug and fix these errors.

### Ticket

The collection name is: `tickets`
ID Column Type: `ULID`

Description: This table contains all of the tickets that are created by users.

Tickets are used to report issues to the moderators.

### WebhookEvent

The collection name is: `webhook_events`
ID Column Type: `String`

Description: This table contains all of the webhook events that are received.

These events are used to track webhooks that are received from Stripe.

Used to deduplicate webhooks.

## Clickhouse

We only have 1 table in Clickhouse called `emote_stats` and it can be seen [here](../clickhouse/emote_stats.sql).

### EmoteStats

This table is used to add up the number of channels adding or removing the emote on a given day.

## Redis

Redis is used for rate limiting and caching.

See the following lua scripts for more information:

- [RateLimit](../apps/api/src/ratelimit/limit.lua)
- [Mutex](../apps/api/src/mutex/mutex.lua)

## Typesense

Almost every table in Mongo has a corresponding collection in Typesense, we then search over these collections using the Typesense API.

## NATs

NATS is used for inter-service communication as a message broker.
