CREATE TYPE "file_set_kind" AS ENUM (
    'TICKET',
    'PROFILE_PICTURE',
    'BADGE',
    'PAINT',
    'EMOTE',
    'PRODUCT',
    'PAGE'
);

CREATE TABLE "file_sets" (
    "id" uuid PRIMARY KEY,
    "kind" file_set_kind NOT NULL,
    "authenticated" bool NOT NULL DEFAULT FALSE,
    "properties" jsonb NOT NULL DEFAULT '{}'
);
