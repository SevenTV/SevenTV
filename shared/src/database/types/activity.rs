use clickhouse::Row;

use super::Table;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct EmoteActivity {
    #[serde(with = "clickhouse::serde::uuid")]
    pub emote_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid::option")]
    pub actor_id: Option<uuid::Uuid>,
    pub kind: EmoteActivityKind,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: time::OffsetDateTime,
}

impl Table for EmoteActivity {
    const TABLE_NAME: &'static str = "emote_activities";
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum EmoteActivityKind {
    Upload = 0,
    Process = 1,
    Edit = 2,
    Merge = 3,
    Delete = 4,
    UndoDelete = 5,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct EmoteSetActivity {
    #[serde(with = "clickhouse::serde::uuid")]
    pub emote_set_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid::option")]
    pub actor_id: Option<uuid::Uuid>,
    pub kind: EmoteSetActivityKind,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: time::OffsetDateTime,
}

impl Table for EmoteSetActivity {
    const TABLE_NAME: &'static str = "emote_set_activities";
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum EmoteSetActivityKind {
    Create = 0,
    Edit = 1,
    Delete = 2,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct UserActivity {
    #[serde(with = "clickhouse::serde::uuid")]
    pub user_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid::option")]
    pub actor_id: Option<uuid::Uuid>,
    pub kind: UserActivityKind,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: time::OffsetDateTime,
}

impl Table for UserActivity {
    const TABLE_NAME: &'static str = "user_activities";
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum UserActivityKind {
    Register = 0,
    Login = 1,
    Logout = 2,
    Edit = 3,
    Delete = 4,
    Merge = 5,
    Ban = 6,
    Unban = 7,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Row)]
pub struct TicketActivity {
    #[serde(with = "clickhouse::serde::uuid")]
    pub ticket_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid::option")]
    pub actor_id: Option<uuid::Uuid>,
    pub kind: TicketActivityKind,
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub timestamp: time::OffsetDateTime,
}

impl Table for TicketActivity {
    const TABLE_NAME: &'static str = "ticket_activities";
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum TicketActivityKind {
    Create = 0,
    Edit = 1,
    Delete = 2,
}
