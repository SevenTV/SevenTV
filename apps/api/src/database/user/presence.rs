use crate::database::Table;

use super::Platform;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserPresence {
    pub user_id: ulid::Ulid,
    pub platform: Platform,
    pub platform_room_id: String,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

impl Table for UserPresence {
    const TABLE_NAME: &'static str = "user_presences";
}
