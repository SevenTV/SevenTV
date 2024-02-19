CREATE TABLE "automod_rules" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "flags" int4 NOT NULL,
    "rules" text[] NOT NULL,
    "added_by_id" uuid, -- Ref: users.id -> On Delete Set Null
    "priority" int2 NOT NULL DEFAULT 0,
    "reason" text NOT NULL
);

CREATE INDEX "automod_rules_added_by_id_index" ON "automod_rules" ("added_by_id");
CREATE UNIQUE INDEX "automod_rules_priority_unique" ON "automod_rules" ("priority");
