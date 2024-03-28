CREATE TYPE "emote_set_kind" AS ENUM ('NORMAL', 'PERSONAL');

CREATE TABLE "emotes" (
    "id" uuid PRIMARY KEY,
    "owner_id" uuid, -- Ref: users.id -> On Delete Set NULL
    "default_name" varchar(128) NOT NULL,
    "tags" text[] NOT NULL DEFAULT '{}',
    "animated" bool NOT NULL DEFAULT FALSE,
    "settings" jsonb NOT NULL DEFAULT '{}',
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE INDEX "emotes_owner_id_index" ON "emotes" ("owner_id");
CREATE UNIQUE INDEX "emotes_ticket_id_unique" ON "emotes" ("ticket_id");

CREATE TABLE "emote_attributions" (
    "emote_id" uuid NOT NULL, -- Ref: emotes.id -> On Delete Cascade
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    PRIMARY KEY ("emote_id", "user_id")
);

CREATE INDEX "emote_attributions_user_id_index" ON "emote_attributions" ("user_id");

CREATE TABLE "emote_files" (
    "emote_id" uuid NOT NULL, -- Ref: emotes.id -> DO NOTHING
    "file_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    PRIMARY KEY ("emote_id", "file_id")
);

CREATE INDEX "emote_files_file_id_index" ON "emote_files" ("file_id");

CREATE TABLE "emote_sets" (
    "id" uuid PRIMARY KEY,
    "owner_id" uuid, -- Ref: users.id -> On Delete Cascade
    "name" varchar(64) NOT NULL,
    "kind" emote_set_kind NOT NULL,
    "flags" int4 NOT NULL,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "tags" text[] NOT NULL DEFAULT '{}'
);

CREATE INDEX "emote_sets_owner_id_index" ON "emote_sets" ("owner_id");

CREATE TABLE "emote_set_emotes" (
    "emote_id" uuid NOT NULL, -- Ref: emotes.id -> On Delete Cascade
    "emote_set_id" uuid NOT NULL, -- Ref: emote_sets.id -> On Delete Cascade
    "added_by_id" uuid, -- Ref: users.id -> On Delete Set Null
    "name" varchar(128) NOT NULL,
    "flags" int4 NOT NULL,
    "added_at" timestamptz NOT NULL DEFAULT 'NOW()',
    PRIMARY KEY ("emote_set_id", "emote_id")
);

CREATE INDEX "emote_set_emotes_added_by_id_index" ON "emote_set_emotes" ("added_by_id");
CREATE INDEX "emote_set_emotes_emote_id_index" ON "emote_set_emotes" ("emote_id");
CREATE UNIQUE INDEX "emote_set_emotes_name_unique" ON "emote_set_emotes" ("emote_set_id", "name");
