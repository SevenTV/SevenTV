CREATE TYPE "page_kind" AS ENUM ('BLOG', 'SUPPORT', 'GENERAL');

CREATE TABLE "pages" (
    "id" uuid PRIMARY KEY,
    "kind" page_kind NOT NULL,
    "title" varchar(255) NOT NULL,
    "slug" varchar(255) NOT NULL,
    "content_md" text NOT NULL,
    "keywords" text[] NOT NULL DEFAULT '{}',
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "settings" jsonb NOT NULL DEFAULT '{}',
    "tags" text[] NOT NULL DEFAULT '{}'
);

CREATE UNIQUE INDEX "pages_slug_unique" ON "pages" ("slug");
CREATE INDEX "pages_kind_index" ON "pages" ("kind");

CREATE TABLE "page_files" (
    "page_id" uuid NOT NULL, -- Ref: pages.id -> DO NOTHING
    "file_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    PRIMARY KEY ("page_id", "file_id")
);

CREATE INDEX "page_files_file_id_index" ON "page_files" ("file_id");

CREATE TABLE "page_authors" (
    "page_id" uuid NOT NULL, -- Ref: pages.id -> On Delete Cascade
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "order" int2 NOT NULL,
    PRIMARY KEY ("page_id", "user_id")
);

CREATE INDEX "page_authors_user_id_index" ON "page_authors" ("user_id");
CREATE UNIQUE INDEX "page_authors_order_unique" ON "page_authors" ("page_id", "order");
