-- Clickhouse SQL

CREATE TABLE emote_activities (
    "emote_id" UUID,
    "actor_id" UUID,
    "kind" Enum(
        'UPLOAD' = 0,
        'EDIT' = 1,
        'MERGE' = 2,
        'DELETE' = 3
    ),
    "timestamp" DateTime DEFAULT NOW()
)
ENGINE = MergeTree
PRIMARY KEY ("emote_id", "actor_id")
ORDER BY ("emote_id", "actor_id", "timestamp");

CREATE TABLE emote_set_activities (
    "emote_set_id" UUID,
    "actor_id" UUID,
    "kind" Enum(
        'CREATE' = 0,
        'EDIT' = 1,
        'DELETE' = 2
    ),
    "timestamp" DateTime DEFAULT NOW()
)
ENGINE = MergeTree
PRIMARY KEY ("emote_set_id", "actor_id")
ORDER BY ("emote_set_id", "actor_id", "timestamp");

CREATE TABLE user_activities (
    "user_id" UUID,
    "actor_id" UUID,
    "kind" Enum(
        'REGISTER' = 0,
        'LOGIN' = 1,
        'LOGOUT' = 2,
        'EDIT' = 3,
        'DELETE' = 4,
        'MERGE' = 5,
        'BAN' = 6,
        'UNBAN' = 7
    ),
    "timestamp" DateTime DEFAULT NOW()
)
ENGINE = MergeTree
PRIMARY KEY ("user_id", "actor_id")
ORDER BY ("user_id", "actor_id", "timestamp");
