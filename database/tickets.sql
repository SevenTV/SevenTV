CREATE TYPE "ticket_priority" AS ENUM ('LOW', 'MEDIUM', 'HIGH', 'URGENT');

CREATE TYPE "ticket_member_kind" AS ENUM ('OP', 'MEMBER', 'STAFF');

CREATE TYPE "ticket_kind" AS ENUM (
    'EMOTE_REPORT',
    'USER_REPORT',
    'BILLING',
    'EMOTE_LISTING_REQUEST',
    'EMOTE_PERSONAL_USE_REQUEST',
    'OTHER'
);

CREATE TYPE "ticket_status" AS ENUM (
    'PENDING',
    'IN_PROGRESS',
    'FIXED',
    'CLOSED'
);

CREATE TABLE "tickets" (
    "id" uuid PRIMARY KEY,
    "kind" ticket_kind NOT NULL,
    "status" ticket_status NOT NULL,
    "priority" ticket_priority NOT NULL,
    "title" text NOT NULL,
    "data" jsonb NOT NULL DEFAULT '{}',
    "tags" text[] NOT NULL DEFAULT '{}',
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()'
);

CREATE INDEX "tickets_status_index" ON "tickets" ("kind", "status");

CREATE TABLE "ticket_files" (
    "ticket_id" uuid NOT NULL, -- Ref: tickets.id -> DO NOTHING
    "file_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    PRIMARY KEY ("ticket_id", "file_id")
);

CREATE INDEX "ticket_files_file_id_index" ON "ticket_files" ("file_id");

CREATE TABLE "ticket_members" (
    "ticket_id" uuid NOT NULL, -- Ref: tickets.id -> On Delete Cascade
    "user_id" uuid NOT NULL,  -- Ref: users.id -> On Delete Cascade
    "kind" ticket_member_kind NOT NULL,
    "notifications" boolean NOT NULL DEFAULT true,
    PRIMARY KEY ("ticket_id", "user_id")
);

CREATE INDEX "ticket_members_user_id_index" ON "ticket_members" ("user_id");
