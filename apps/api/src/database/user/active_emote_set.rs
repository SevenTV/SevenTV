use ulid::Ulid;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserActiveEmoteSet {
	pub user_id: Ulid,
	pub emote_set_id: Ulid,
	pub priority: i16,
}
