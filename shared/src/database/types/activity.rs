use postgres_types::{FromSql, ToSql};

use super::Table;

#[derive(Debug, Clone, postgres_from_row::FromRow)]
pub struct EmoteActivity {
    pub id: ulid::Ulid,
    pub emote_id: ulid::Ulid,
    pub actor_id: Option<ulid::Ulid>,
    pub kind: EmoteActivityKind,
    #[from_row(from_fn = "scuffle_utils::database::json")]
    pub data: EmoteActivityData,
}

impl Table for EmoteActivity {
    const TABLE_NAME: &'static str = "emote_activities";
}

#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name = "emote_activity_kind")]
pub enum EmoteActivityKind {
    #[postgres(name = "UPLOAD")]
    Upload,
    #[postgres(name = "EDIT")]
    Edit,
    #[postgres(name = "MERGE")]
    Merge,
    #[postgres(name = "DELETE")]
    Delete,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EmoteActivityData {}

#[derive(Debug, Clone, postgres_from_row::FromRow)]
pub struct EmoteSetActivity {
    pub id: ulid::Ulid,
    pub emote_set_id: ulid::Ulid,
    pub actor_id: Option<ulid::Ulid>,
    pub kind: EmoteSetActivityKind,
    #[from_row(from_fn = "scuffle_utils::database::json")]
    pub data: EmoteSetActivityData,
}

impl Table for EmoteSetActivity {
    const TABLE_NAME: &'static str = "emote_set_activities";
}

#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name = "emote_set_activity_kind")]
pub enum EmoteSetActivityKind {
    #[postgres(name = "CREATE")]
    Create,
    #[postgres(name = "EDIT")]
    Edit,
    #[postgres(name = "DELETE")]
    Delete,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EmoteSetActivityData {}

#[derive(Debug, Clone, postgres_from_row::FromRow)]
pub struct UserActivity {
    pub id: ulid::Ulid,
    pub user_id: ulid::Ulid,
    pub actor_id: Option<ulid::Ulid>,
    pub kind: UserActivityKind,
    #[from_row(from_fn = "scuffle_utils::database::json")]
    pub data: UserActivityData,
}

impl Table for UserActivity {
    const TABLE_NAME: &'static str = "user_activities";
}

#[derive(Debug, Clone, ToSql, FromSql)]
#[postgres(name = "user_activity_kind")]
pub enum UserActivityKind {
    #[postgres(name = "LOGIN")]
    Login,
    #[postgres(name = "LOGOUT")]
    Logout,
    #[postgres(name = "REGISTER")]
    Register,
    #[postgres(name = "EDIT")]
    Edit,
    #[postgres(name = "MERGE")]
    Merge,
    #[postgres(name = "DELETE")]
    Delete,
    #[postgres(name = "BAN")]
    Ban,
    #[postgres(name = "UNBAN")]
    Unban,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UserActivityData {}
