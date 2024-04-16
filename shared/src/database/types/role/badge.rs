use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct RoleBadge {
	pub role_id: ulid::Ulid,
	pub badge_id: ulid::Ulid,
}

impl Table for RoleBadge {
	const TABLE_NAME: &'static str = "role_badges";
}
