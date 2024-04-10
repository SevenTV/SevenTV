CREATE TABLE "roles" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "description" text DEFAULT NULL,
    "data" jsonb NOT NULL DEFAULT '{}',
    "priority" int2 NOT NULL DEFAULT 0,
    "hoist" boolean NOT NULL DEFAULT false,
    "color" int4 NOT NULL DEFAULT 0,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "tags" text[] NOT NULL DEFAULT '{}'
);

CREATE UNIQUE INDEX "roles_priority_unique" ON "roles" ("priority");

CREATE TABLE "role_badges" (
    "role_id" uuid NOT NULL, -- Ref: roles.id -> On Delete Cascade
    "badge_id" uuid NOT NULL, -- Ref: badges.id -> DO NOTHING
    PRIMARY KEY ("role_id", "badge_id")
);

CREATE INDEX "role_badges_badge_id_index" ON "role_badges" ("badge_id");

CREATE TABLE "role_emote_sets" (
    "role_id" uuid NOT NULL, -- Ref: roles.id -> On Delete Cascade
    "emote_set_id" uuid NOT NULL, -- Ref: emote_sets.id -> DO NOTHING
    PRIMARY KEY ("role_id", "emote_set_id")
);

CREATE INDEX "role_emote_sets_emote_set_id_index" ON "role_emote_sets" ("emote_set_id");

CREATE TABLE "role_paints" (
    "role_id" uuid NOT NULL, -- Ref: roles.id -> On Delete Cascade
    "paint_id" uuid NOT NULL, -- Ref: paints.id -> DO NOTHING
    PRIMARY KEY ("role_id", "paint_id")
);

CREATE INDEX "role_paints_paint_id_index" ON "role_paints" ("paint_id");
