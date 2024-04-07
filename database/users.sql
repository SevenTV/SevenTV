CREATE TYPE "two_fa" AS ("secret" bytea, "codes" int4 []);

CREATE TABLE "users" (
    "id" uuid PRIMARY KEY,
    "email" varchar(255) DEFAULT NULL,
    "email_verified" boolean NOT NULL DEFAULT false,
    "password_hash" varchar(255) DEFAULT NULL,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "notification_settings" jsonb NOT NULL DEFAULT '{}',
    "message_settings" jsonb NOT NULL DEFAULT '{}',
    "editor_settings" jsonb NOT NULL DEFAULT '{}',
    "two_fa" two_fa DEFAULT NULL,
    "active_badge_id" uuid DEFAULT NULL, -- Ref: badges.id -> On Delete Set NULL
    "active_paint_id" uuid DEFAULT NULL, -- Ref: paints.id -> On Delete Set NULL
    "pending_profile_picture_file_set_id" uuid DEFAULT NULL, -- file_sets.id -> On Delete Set NULL (Deferable)
    "active_profile_picture_file_set_id" uuid DEFAULT NULL, -- file_sets.id -> On Delete Set NULL (Deferable)

    -- Cached fields for quick access
    "entitled_cache_role_ids" uuid [] NOT NULL DEFAULT '{}',
    "entitled_cache_badge_ids" uuid [] NOT NULL DEFAULT '{}',
    "entitled_cache_paint_ids" uuid [] NOT NULL DEFAULT '{}',
    "entitled_cache_emote_set_ids" uuid [] NOT NULL DEFAULT '{}',
    -- The following fields are used to invalidate the cache when the user's entitlements change
    "entitled_cache_invalidated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "entitled_cache_invalidate_role_ids" uuid [] NOT NULL DEFAULT '{}',
    "entitled_cache_invalidate_product_ids" uuid [] NOT NULL DEFAULT '{}'
);

CREATE INDEX "users_entitled_cache_role_ids_index" ON "users" USING YBGIN ("entitled_cache_role_ids");
CREATE INDEX "users_entitled_cache_badge_ids_index" ON "users" USING YBGIN ("entitled_cache_badge_ids");
CREATE INDEX "users_entitled_cache_paint_ids_index" ON "users" USING YBGIN ("entitled_cache_paint_ids");
CREATE INDEX "users_entitled_cache_emote_set_ids_index" ON "users" USING YBGIN ("entitled_cache_emote_set_ids");
CREATE INDEX "users_entitled_cache_invalidate_role_ids_index" ON "users" USING YBGIN ("entitled_cache_invalidate_role_ids");
CREATE INDEX "users_entitled_cache_invalidate_product_ids_index" ON "users" USING YBGIN ("entitled_cache_invalidate_product_ids");

CREATE INDEX "users_active_badge_id_index" ON "users" ("active_badge_id");
CREATE INDEX "users_active_paint_id_index" ON "users" ("active_paint_id");
CREATE INDEX "users_active_profile_picture_file_set_id_index" ON "users" ("active_profile_picture_file_set_id");

CREATE TABLE "user_sessions" (
    "id" uuid PRIMARY KEY,
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "expires_at" timestamptz NOT NULL,
    "last_used_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE TYPE "platform" AS ENUM ('DISCORD', 'TWITCH', 'GOOGLE', 'KICK');

CREATE TABLE "user_connections" (
    "id" uuid PRIMARY KEY,
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "main_connection" boolean NOT NULL DEFAULT false,
    "platform_kind" platform NOT NULL,
    "platform_id" varchar(64) NOT NULL,
    "platform_username" varchar(255) NOT NULL,
    "platform_display_name" varchar(255) NOT NULL,
    "platform_avatar_url" varchar(255),
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "allow_login" boolean NOT NULL DEFAULT true
);

CREATE INDEX "user_connections_user_id_index" ON "user_connections" ("user_id");
CREATE UNIQUE INDEX "user_connections_platform_id_index" ON "user_connections" ("platform_kind", "platform_id");
CREATE UNIQUE INDEX "user_connections_main_connection_index" ON "user_connections" ("user_id") WHERE "main_connection";

CREATE TABLE "user_follows" (
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "followed_user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "followed_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "data" jsonb NOT NULL DEFAULT '{}',
    PRIMARY KEY ("user_id", "followed_user_id")
);

CREATE INDEX "user_follows_followed_user_id_index" ON "user_follows" ("followed_user_id", "followed_at");

CREATE TABLE "user_blocks" (
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "blocked_user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "blocked_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "reason" varchar(255),
    PRIMARY KEY ("user_id", "blocked_user_id")
);

CREATE INDEX "user_blocks_blocked_user_id_index" ON "user_blocks" ("blocked_user_id");

CREATE TABLE "user_bans" (
    "id" uuid PRIMARY KEY,
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "reason" text NOT NULL,
    "expires_at" timestamptz,
    "kind" int4 NOT NULL,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "created_by_id" uuid -- Ref: users.id -> On Delete Set NULL
);

CREATE INDEX "user_bans_user_id_index" ON "user_bans" ("user_id");
CREATE INDEX "user_bans_created_by_id_index" ON "user_bans" ("created_by_id");

CREATE TYPE "user_editor_state" AS ENUM ('ACTIVE', 'INVITED', 'BLOCKED_INVITE');

CREATE TABLE "user_editors" (
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "editor_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "state" user_editor_state NOT NULL DEFAULT 'INVITED',
    "permissions" int8 NOT NULL DEFAULT 0,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    PRIMARY KEY ("user_id", "editor_id")
);

CREATE INDEX "user_editors_editor_id_index" ON "user_editors" ("editor_id");

CREATE TABLE "user_active_emote_sets" (
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "emote_set_id" uuid NOT NULL, -- Ref: emote_sets.id -> On Delete Cascade
    "priority" int2 NOT NULL,
    PRIMARY KEY ("user_id", "emote_set_id")
);

CREATE INDEX "user_active_emote_sets_user_id_index" ON "user_active_emote_sets" ("user_id");
CREATE UNIQUE INDEX "user_active_emote_sets_user_id_priority_index" ON "user_active_emote_sets" ("user_id", "priority");
CREATE INDEX "user_active_emote_sets_emote_set_id_index" ON "user_active_emote_sets" ("emote_set_id");

CREATE TABLE "user_roles" (
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "role_id" uuid NOT NULL, -- Ref: roles.id -> On Delete Cascade
    "added_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "added_by_id" uuid, -- Ref: users.id -> On Delete Set Null
    PRIMARY KEY ("user_id", "role_id")
);

CREATE INDEX "user_roles_role_id_index" ON "user_roles" ("role_id");
CREATE INDEX "user_roles_added_by_id_index" ON "user_roles" ("added_by_id");
