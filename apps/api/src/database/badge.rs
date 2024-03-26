use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Badge {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for Badge {
	const TABLE_NAME: &'static str = "badges";
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct BadgeFile {
	pub badge_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
}

impl Table for BadgeFile {
	const TABLE_NAME: &'static str = "badge_files";
}
