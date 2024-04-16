use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct UserPaint {
	pub user_id: ulid::Ulid,
	pub paint_id: ulid::Ulid,
}

impl Table for UserPaint {
	const TABLE_NAME: &'static str = "user_paints";
}
