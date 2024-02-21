CREATE TYPE "two_fa" AS ("secret" bytea, "codes" uuid []);

CREATE TABLE "users" (
    "id" uuid PRIMARY KEY,
    "email" varchar(255),
    "email_verified" boolean NOT NULL DEFAULT false,
    "password" varchar(255),
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "notification_settings" jsonb NOT NULL DEFAULT '{}',
    "message_settings" jsonb NOT NULL DEFAULT '{}',
    "editor_settings" jsonb NOT NULL DEFAULT '{}',
    "two_fa" two_fa,
    "active_badge_id" uuid, -- Ref: badges.id -> On Delete Set NULL
    "active_paint_id" uuid, -- Ref: paints.id -> On Delete Set NULL
    "active_profile_picture_id" uuid NOT NULL, -- user_profile_pictures.id -> On Delete Set NULL (Deferable)

    -- Cached fields for quick access
    "entitled_cache_role_ids" uuid [] NOT NULL,
    "entitled_cache_badge_ids" uuid [] NOT NULL,
    "entitled_cache_paint_ids" uuid [] NOT NULL,
    "entitled_cache_emote_set_ids" uuid [] NOT NULL,
    -- The following fields are used to invalidate the cache when the user's entitlements change
    "entitled_cache_invalidated_at" timestamptz NOT NULL,
    "entitled_cache_invalidate_role_ids" uuid [] NOT NULL,
    "entitled_cache_invalidate_product_ids" uuid [] NOT NULL
);

CREATE INDEX "users_entitled_cache_role_ids_index" ON "users" USING YBGIN ("entitled_cache_role_ids");
CREATE INDEX "users_entitled_cache_badge_ids_index" ON "users" USING YBGIN ("entitled_cache_badge_ids");
CREATE INDEX "users_entitled_cache_paint_ids_index" ON "users" USING YBGIN ("entitled_cache_paint_ids");
CREATE INDEX "users_entitled_cache_emote_set_ids_index" ON "users" USING YBGIN ("entitled_cache_emote_set_ids");
CREATE INDEX "users_entitled_cache_invalidate_role_ids_index" ON "users" USING YBGIN ("entitled_cache_invalidate_role_ids");
CREATE INDEX "users_entitled_cache_invalidate_product_ids_index" ON "users" USING YBGIN ("entitled_cache_invalidate_product_ids");

CREATE INDEX "users_active_badge_id_index" ON "users" ("active_badge_id");
CREATE INDEX "users_active_paint_id_index" ON "users" ("active_paint_id");
CREATE INDEX "users_active_profile_picture_id_index" ON "users" ("active_profile_picture_id");

CREATE TYPE "connection_platform" AS ENUM ('DISCORD', 'TWITCH', 'YOUTUBE');

CREATE TABLE "user_connections" (
    "id" uuid PRIMARY KEY,
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "platform" connection_platform NOT NULL,
    "platform_id" varchar(64) NOT NULL,
    "platform_username" varchar(255) NOT NULL,
    "platform_avatar_url" varchar(255),
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "allow_login" boolean NOT NULL DEFAULT true
);

CREATE INDEX "user_connections_user_id_index" ON "user_connections" ("user_id");
CREATE UNIQUE INDEX "user_connections_platform_id_index" ON "user_connections" ("platform", "platform_id");

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

CREATE TABLE "user_profile_pictures" (
    "id" uuid PRIMARY KEY,
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE INDEX "user_profile_pictures_user_id_index" ON "user_profile_pictures" ("user_id");

CREATE TABLE "user_profile_picture_files" (
    "user_profile_picture_id" uuid NOT NULL, -- Ref: user_profile_pictures.id -> DO NOTHING
    "file_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    -- TODO add more fields to describe the file, size width height etc
    PRIMARY KEY ("user_profile_picture_id", "file_id")
);

CREATE INDEX "user_profile_picture_files_file_id_index" ON "user_profile_picture_files" ("file_id");

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
