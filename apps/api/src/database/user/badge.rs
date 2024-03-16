use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserBadge {
	pub user_id: ulid::Ulid,
	pub badge_id: ulid::Ulid,
}

impl Table for UserBadge {
	const TABLE_NAME: &'static str = "user_badges";
}
