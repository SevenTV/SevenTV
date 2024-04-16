use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserRoles {
	pub user_id: ulid::Ulid,
	pub role_id: ulid::Ulid,
	pub added_at: chrono::DateTime<chrono::Utc>,
	pub added_by_id: Option<ulid::Ulid>,
}

impl Table for UserRoles {
	const TABLE_NAME: &'static str = "user_roles";
}
