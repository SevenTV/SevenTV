CREATE TYPE "product_type" AS ENUM ('BASE', 'ADDON', 'BUNDLE');

CREATE TYPE "product_visibility" AS ENUM ('PUBLIC', 'UNLISTED');

CREATE TABLE "products" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "description" text,
    "tags" text[] NOT NULL DEFAULT '{}',
    "enabled" boolean NOT NULL DEFAULT true,
    "remaining_stock" int4, -- NULL means unlimited
    "type" product_type NOT NULL,
    "rank" int2, -- NULL means not featured
    "visibility" product_visibility NOT NULL,
    "data" jsonb NOT NULL,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE UNIQUE INDEX "products_rank_unique" ON "products" ("rank") WHERE "rank" IS NOT NULL;

CREATE TABLE "product_association_products" (
    "product_id" uuid NOT NULL, -- Ref: products.id -> On Delete Cascade
    "associated_product_id" uuid NOT NULL, -- Ref: products.id -> DO NOTHING
    PRIMARY KEY ("product_id", "associated_product_id")
);

CREATE INDEX "product_association_products_associated_product_id_index" ON "product_association_products" ("associated_product_id");

CREATE TABLE "product_association_roles" (
    "product_id" uuid NOT NULL, -- Ref: products.id -> On Delete Cascade
    "associated_role_id" uuid NOT NULL, -- Ref: roles.id -> DO NOTHING
    PRIMARY KEY ("product_id", "associated_role_id")
);

CREATE INDEX "product_association_roles_associated_role_id_index" ON "product_association_roles" ("associated_role_id");

CREATE TABLE "product_association_emote_sets" (
    "product_id" uuid NOT NULL, -- Ref: products.id -> On Delete Cascade
    "associated_emote_set_id" uuid NOT NULL, -- Ref: emote_sets.id -> DO NOTHING
    PRIMARY KEY ("product_id", "associated_emote_set_id")
);

CREATE INDEX "product_association_emote_sets_associated_emote_set_id_index" ON "product_association_emote_sets" ("associated_emote_set_id");

CREATE TABLE "product_association_badges" (
    "product_id" uuid NOT NULL, -- Ref: products.id -> On Delete Cascade
    "associated_badge_id" uuid NOT NULL, -- Ref: badges.id -> DO NOTHING
    PRIMARY KEY ("product_id", "associated_badge_id")
);

CREATE INDEX "product_association_badges_associated_badge_id_index" ON "product_association_badges" ("associated_badge_id");

CREATE TABLE "product_association_paints" (
    "product_id" uuid NOT NULL, -- Ref: products.id -> On Delete Cascade
    "associated_paint_id" uuid NOT NULL, -- Ref: paints.id -> DO NOTHING
    PRIMARY KEY ("product_id", "associated_paint_id")
);

CREATE INDEX "product_association_paints_associated_paint_id_index" ON "product_association_paints" ("associated_paint_id");

CREATE TYPE "product_code_type" AS ENUM ('REDEEM', 'DISCOUNT');

CREATE TABLE "product_codes" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "code" varchar(64) NOT NULL,
    "description" text,
    "tags" text[] NOT NULL DEFAULT '{}',
    "data" jsonb NOT NULL,
    "enabled" boolean NOT NULL DEFAULT true,
    "remaining_uses" int4, -- NULL means unlimited
    "type" product_code_type NOT NULL,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE TABLE "product_code_association_product" (
    "product_code_id" uuid NOT NULL, -- Ref: product_codes.id -> On Delete Cascade
    "associated_product_id" uuid NOT NULL, -- Ref: products.id -> DO NOTHING
    PRIMARY KEY ("product_code_id", "associated_product_id")
);

CREATE INDEX "product_code_association_product_associated_product_id_index" ON "product_code_association_product" ("associated_product_id");

CREATE TYPE "product_purchase_status" AS ENUM ('PENDING', 'COMPLETED', 'REFUNDED', 'FAILED');

CREATE TABLE "product_purchases" (
    "id" uuid PRIMARY KEY,
    "product_id" uuid NOT NULL, -- Ref: products.id -> DO NOTHING
    "user_id" uuid, -- Ref: users.id -> Set Null
    "recipient_id" uuid, -- Ref: users.id -> Set Null
    "data" jsonb NOT NULL,
    "status" product_purchase_status NOT NULL,
    "is_subscription" boolean NOT NULL DEFAULT false,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE INDEX "product_purchases_product_id_index" ON "product_purchases" ("product_id");
CREATE INDEX "product_purchases_user_id_index" ON "product_purchases" ("user_id");
CREATE INDEX "product_purchases_recipient_id_index" ON "product_purchases" ("recipient_id");
CREATE INDEX "product_purchases_status_index" ON "product_purchases" ("status");

CREATE TYPE "product_subscription_status" AS ENUM ('ACTIVE', 'INACTIVE', 'FAILED');

CREATE TABLE "product_subscriptions" (
    "product_id" uuid NOT NULL, -- Ref: products.id -> DO NOTHING
    "user_id" uuid NOT NULL,
    "data" jsonb NOT NULL,
    "status" product_subscription_status NOT NULL,
    "next_payment_due" timestamptz,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    PRIMARY KEY ("product_id", "user_id")
);

CREATE INDEX "product_subscriptions_product_id_index" ON "product_subscriptions" ("product_id");
CREATE INDEX "product_subscriptions_user_id_index" ON "product_subscriptions" ("user_id");
CREATE INDEX "product_subscriptions_status_index" ON "product_subscriptions" ("status");
CREATE INDEX "product_subscriptions_next_payment_due_index" ON "product_subscriptions" ("next_payment_due");
