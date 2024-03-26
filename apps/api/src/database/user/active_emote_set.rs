use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserActiveEmoteSet {
	pub user_id: ulid::Ulid,
	pub emote_set_id: ulid::Ulid,
	pub priority: i16,
}

impl Table for UserActiveEmoteSet {
	const TABLE_NAME: &'static str = "user_active_emote_sets";
}
