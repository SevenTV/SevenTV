use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserEmoteSet {
	pub user_id: ulid::Ulid,
	pub emote_set_id: ulid::Ulid,
}

impl Table for UserEmoteSet {
	const TABLE_NAME: &'static str = "user_emote_sets";
}
