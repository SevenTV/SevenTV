CREATE TABLE "paints" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "description" text,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "tags" text[] NOT NULL DEFAULT '{}'
);

CREATE TABLE "paint_files" (
    "paint_id" uuid NOT NULL, -- Ref: paints.id -> DO NOTHING
    "file_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    PRIMARY KEY ("paint_id", "file_id")
);

CREATE INDEX "paint_files_file_id_index" ON "paint_files" ("file_id");

