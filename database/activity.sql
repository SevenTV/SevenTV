-- Clickhouse SQL

CREATE TABLE emote_activities (
    "emote_id" UUID,
    "actor_id" Nullable(UUID),
    "kind" Enum8(
        'UPLOAD' = 0,
        'PROCESS' = 1,
        'EDIT' = 2,
        'MERGE' = 3,
        'DELETE' = 4,
        'UNDO_DELETE' = 5
    ),
    "timestamp" DateTime64(3) DEFAULT NOW()
)
ENGINE = MergeTree
ORDER BY ("emote_id", "kind", "timestamp");

CREATE TABLE emote_set_activities (
    "emote_set_id" UUID,
    "actor_id" Nullable(UUID),
    "kind" Enum8(
        'CREATE' = 0,
        'EDIT' = 1,
        'DELETE' = 2
    ),
    "timestamp" DateTime64(3) DEFAULT NOW()
)
ENGINE = MergeTree
ORDER BY ("emote_set_id", "kind", "timestamp");

CREATE TABLE user_activities (
    "user_id" UUID,
    "actor_id" Nullable(UUID),
    "kind" Enum8(
        'REGISTER' = 0,
        'LOGIN' = 1,
        'LOGOUT' = 2,
        'EDIT' = 3,
        'DELETE' = 4,
        'MERGE' = 5,
        'BAN' = 6,
        'UNBAN' = 7
    ),
    "timestamp" DateTime64(3) DEFAULT NOW()
)
ENGINE = MergeTree
ORDER BY ("user_id", "kind", "timestamp");

CREATE TABLE ticket_activities (
    "ticket_id" UUID,
    "actor_id" Nullable(UUID),
    "kind" Enum8(
        'CREATE' = 0,
        'EDIT' = 1,
        'DELETE' = 2
    ),
    "timestamp" DateTime64(3) DEFAULT NOW()
)
ENGINE = MergeTree
ORDER BY ("ticket_id", "kind", "timestamp");
