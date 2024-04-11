CREATE TYPE emote_activity_kind AS ENUM (
    'UPLOAD',
    'EDIT',
    'MERGE',
    'DELETE'
);

CREATE TABLE emote_activities (
    "id" uuid PRIMARY KEY,
    "emote_id" uuid NOT NULL, -- Ref: emotes.id -> On Delete Cascade
    "actor_id" uuid, -- Ref: users.id -> On Delete Set Null
    "kind" emote_activity_kind NOT NULL,
    "data" jsonb NOT NULL DEFAULT '{}'
);

CREATE INDEX emote_activities_emote_id_index ON emote_activities ("emote_id");
CREATE INDEX emote_activities_actor_id_index ON emote_activities ("actor_id");

CREATE TYPE emote_set_activity_kind AS ENUM (
    'CREATE',
    'EDIT',
    'DELETE'
);

CREATE TABLE emote_set_activities (
    "id" uuid PRIMARY KEY,
    "emote_set_id" uuid NOT NULL, -- Ref: emote_sets.id -> On Delete Cascade
    "actor_id" uuid, -- Ref: users.id -> On Delete Set Null
    "kind" emote_set_activity_kind NOT NULL,
    "data" jsonb NOT NULL DEFAULT '{}'
);

CREATE INDEX emote_set_activities_emote_set_id_index ON emote_set_activities ("emote_set_id");
CREATE INDEX emote_set_activities_actor_id_index ON emote_set_activities ("actor_id");

CREATE TYPE user_activity_kind AS ENUM (
    'LOGIN',
    'LOGOUT',
    'REGISTER',
    'EDIT',
    'MERGE',
    'DELETE',
    'BAN',
    'UNBAN'
);

CREATE TABLE user_activities (
    "id" uuid PRIMARY KEY,
    "user_id" uuid NOT NULL, -- Ref: users.id -> On Delete Cascade
    "actor_id" uuid, -- Ref: users.id -> On Delete Set Null
    "kind" user_activity_kind NOT NULL,
    "data" jsonb NOT NULL DEFAULT '{}'
);

CREATE INDEX user_activities_user_id_index ON user_activities ("user_id");
CREATE INDEX user_activities_actor_id_index ON user_activities ("actor_id");
