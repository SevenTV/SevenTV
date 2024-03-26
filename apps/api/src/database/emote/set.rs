use postgres_types::{FromSql, ToSql};

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct EmoteSet {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub name: String,
	pub kind: EmoteSetKind,
	pub tags: Vec<String>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: EmoteSetSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for EmoteSet {
	const TABLE_NAME: &'static str = "emote_sets";
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "emote_set_kind")]
pub enum EmoteSetKind {
	#[default]
	#[postgres(name = "NORMAL")]
	Normal,
	#[postgres(name = "PERSONAL")]
	Personal,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct EmoteSetSettings {}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct EmoteSetEmote {
	pub emote_set_id: ulid::Ulid,
	pub emote_id: ulid::Ulid,
	pub added_by_id: Option<ulid::Ulid>,
	pub name: String,
	pub flags: i64,
	pub added_at: chrono::DateTime<chrono::Utc>,
}
