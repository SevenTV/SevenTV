use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Paint {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: PaintData,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for Paint {
	const TABLE_NAME: &'static str = "paints";
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct PaintFile {
	pub paint_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
}

impl Table for PaintFile {
	const TABLE_NAME: &'static str = "paint_files";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct PaintData {}
