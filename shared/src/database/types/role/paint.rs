use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct RolePaint {
	pub role_id: ulid::Ulid,
	pub paint_id: ulid::Ulid,
}

impl Table for RolePaint {
	const TABLE_NAME: &'static str = "role_paints";
}
