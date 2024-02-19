CREATE TYPE "file_kind" AS ENUM (
    'PROFILE_PICTURE',
    'BADGE',
    'PAINT',
    'EMOTE',
    'PRODUCT',
    'TICKET',
    'PAGE'
);

CREATE TABLE "files" (
    "id" uuid PRIMARY KEY,
    "owner_id" uuid, -- Ref: users.id -> On Delete Set Null
    "kind" file_kind NOT NULL,
    "path" varchar(255) NOT NULL,
    "mime_type" varchar(64) NOT NULL,
    "properties" jsonb NOT NULL DEFAULT '{}'
);

CREATE INDEX "files_owner_id_index" ON "files" ("owner_id");
