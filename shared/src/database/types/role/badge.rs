use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct RoleBadge {
	pub role_id: ulid::Ulid,
	pub badge_id: ulid::Ulid,
}

impl Table for RoleBadge {
	const TABLE_NAME: &'static str = "role_badges";
}
