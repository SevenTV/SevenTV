CREATE TABLE "paints" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "description" text DEFAULT NULL,
    "tags" text[] NOT NULL DEFAULT '{}',
    "data" jsonb NOT NULL,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE TABLE "paint_file_sets" (
    "paint_id" uuid NOT NULL, -- Ref: paints.id -> DO NOTHING
    "file_set_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    PRIMARY KEY ("paint_id", "file_set_id")
);

CREATE INDEX "paint_file_sets_file_set_id_index" ON "paint_file_sets" ("file_set_id");
CREATE INDEX "paint_file_sets_paint_id_index" ON "paint_file_sets" ("paint_id");
