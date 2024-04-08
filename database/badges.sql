CREATE TABLE "badges" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "description" text DEFAULT NULL,
    "tags" text[] NOT NULL DEFAULT '{}',
    "file_set_id" uuid NOT NULL, -- Ref: file_sets.id -> On Delete Cascade
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);
