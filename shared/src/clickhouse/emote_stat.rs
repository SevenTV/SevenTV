use super::ClickhouseCollection;
use crate::database::emote::EmoteId;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, clickhouse::Row)]
pub struct EmoteStat {
	#[serde(with = "clickhouse::serde::time::date")]
	pub date: time::Date,
	pub emote_id: EmoteId,
	pub count: i32,
}

impl ClickhouseCollection for EmoteStat {
	const COLLECTION_NAME: &'static str = "emote_stats";
}
