use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct EmoteAttribution {
	pub emote_id: ulid::Ulid,
	pub user_id: ulid::Ulid,
	pub added_at: chrono::DateTime<chrono::Utc>,
}

impl Table for EmoteAttribution {
	const TABLE_NAME: &'static str = "emote_attributions";
}
