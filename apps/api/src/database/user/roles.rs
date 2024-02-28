use ulid::Ulid;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct UserRoles {
	pub user_id: Ulid,
	pub role_id: Ulid,
	pub added_at: chrono::DateTime<chrono::Utc>,
	pub added_by_id: Option<Ulid>,
}
