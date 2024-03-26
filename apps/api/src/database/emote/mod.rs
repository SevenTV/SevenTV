mod attribution;
mod file;
mod set;

pub use attribution::*;
pub use file::*;
pub use set::*;

use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Emote {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub default_name: String,
	pub tags: Vec<String>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: EmoteSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for Emote {
	const TABLE_NAME: &'static str = "emotes";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct EmoteSettings {}
