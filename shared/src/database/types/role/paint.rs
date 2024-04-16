use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct RolePaint {
	pub role_id: ulid::Ulid,
	pub paint_id: ulid::Ulid,
}

impl Table for RolePaint {
	const TABLE_NAME: &'static str = "role_paints";
}
