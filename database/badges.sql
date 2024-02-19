CREATE TABLE "badges" (
    "id" uuid PRIMARY KEY,
    "name" varchar(64) NOT NULL,
    "description" text,
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "tags" text[] NOT NULL DEFAULT '{}'
);

CREATE TABLE "badge_files" (
    "badge_id" uuid NOT NULL, -- Ref: badges.id -> DO NOTHING
    "file_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    PRIMARY KEY ("badge_id", "file_id")
);

CREATE INDEX "badge_files_file_id_index" ON "badge_files" ("file_id");
CREATE INDEX "badge_files_badge_id_index" ON "badge_files" ("badge_id");
