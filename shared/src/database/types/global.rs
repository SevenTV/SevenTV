use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct GlobalConfig {
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub alerts: GlobalConfigAlerts,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for GlobalConfig {
	const TABLE_NAME: &'static str = "global_config";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct GlobalConfigAlerts {}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct GlobalActiveEmoteSet {
	pub emote_set_id: ulid::Ulid,
	pub priority: i16,
}

impl Table for GlobalActiveEmoteSet {
	const TABLE_NAME: &'static str = "global_active_emote_sets";
}
