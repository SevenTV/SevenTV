CREATE TABLE "global" (
    "alerts" jsonb NOT NULL DEFAULT '{}',
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE TABLE "global_active_emote_sets" (
    "emote_set_id" uuid NOT NULL, -- Ref: emote_sets.id -> On Delete Cascade
    "priority" int2 NOT NULL,
    PRIMARY KEY ("emote_set_id")
);

CREATE UNIQUE INDEX "global_active_emote_sets_priority_unique" ON "global_active_emote_sets" ("priority");
CREATE INDEX "global_active_emote_sets_emote_set_id_index" ON "global_active_emote_sets" ("emote_set_id");
