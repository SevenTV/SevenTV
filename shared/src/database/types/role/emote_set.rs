use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct RoleEmoteSet {
	pub role_id: ulid::Ulid,
	pub emote_set_id: ulid::Ulid,
}

impl Table for RoleEmoteSet {
	const TABLE_NAME: &'static str = "role_emote_sets";
}
