CREATE TYPE "ticket_priority" AS ENUM ('LOW', 'MEDIUM', 'HIGH', 'URGENT');

CREATE TYPE "ticket_member_kind" AS ENUM ('OP', 'MEMBER', 'STAFF');

CREATE TYPE "ticket_kind" AS ENUM (
    'EMOTE_REPORT',
    'USER_REPORT',
    'BUG_REPORT',
    'FEATURE_REQUEST',
    'BILLING',
    'EMOTE_LISTING_REQUEST',
    'OTHER'
);

CREATE TYPE "ticket_state" AS ENUM (
    'PENDING_STAFF',
    'PENDING_USER',
    'IN_PROGRESS',
    'FIXED',
    'PLANNED',
    'CLOSED'
);

CREATE TABLE "tickets" (
    "id" uuid PRIMARY KEY,
    "kind" ticket_kind NOT NULL,
    "state" ticket_state NOT NULL,
    "priority" ticket_priority NOT NULL,
    "title" varchar(64) NOT NULL,
    "data" jsonb NOT NULL DEFAULT '{}',
    "updated_at" timestamptz NOT NULL DEFAULT 'NOW()',
    "locked" boolean NOT NULL DEFAULT false,
    "private" boolean NOT NULL DEFAULT false,
    "closed" boolean NOT NULL DEFAULT false,
    "tags" text[] NOT NULL DEFAULT '{}'
);

CREATE INDEX "tickets_state_index" ON "tickets" ("kind", "state");

CREATE TABLE "ticket_files" (
    "ticket_id" uuid NOT NULL, -- Ref: tickets.id -> DO NOTHING
    "file_id" uuid NOT NULL, -- Ref: files.id -> DO NOTHING
    PRIMARY KEY ("ticket_id", "file_id")
);

CREATE INDEX "ticket_files_file_id_index" ON "ticket_files" ("file_id");

CREATE TABLE "ticket_members" (
    "ticket_id" uuid NOT NULL, -- Ref: tickets.id -> On Delete Cascade
    "user_id" uuid NOT NULL,  -- Ref: users.id -> On Delete Cascade
    "kind" support_ticket_member_kind NOT NULL,
    "voted" boolean NOT NULL DEFAULT false,
    "notifications" boolean NOT NULL DEFAULT true,
    PRIMARY KEY ("ticket_id", "user_id")
);

CREATE INDEX "ticket_members_user_id_index" ON "ticket_members" ("user_id");
