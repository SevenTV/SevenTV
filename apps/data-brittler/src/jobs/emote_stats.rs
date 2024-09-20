use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use anyhow::Context;
use shared::clickhouse::emote_stat::EmoteStat;
use shared::database::emote::EmoteId;
use time::OffsetDateTime;

pub struct RunInput {
	pub clickhouse: clickhouse::Client,
	pub truncate: bool,
	pub emote_stats: BTreeMap<(EmoteId, time::Date), i32>,
	pub true_emote_stats: HashMap<EmoteId, i32>,
}

pub async fn run(input: RunInput) -> anyhow::Result<()> {
	let RunInput {
		clickhouse,
		mut emote_stats,
		true_emote_stats,
		truncate,
	} = input;

	clickhouse
		.query(include_str!("../../../../clickhouse/emote_stats.sql"))
		.execute()
		.await
		.context("create table")?;

	if truncate {
		clickhouse
			.query("TRUNCATE TABLE emote_stats")
			.execute()
			.await
			.context("truncate table")?;
	}

	let mut inserter = clickhouse
		.insert("emote_stats")
		.context("insert")?
		.with_timeouts(Some(Duration::from_secs(120)), Some(Duration::from_secs(120)));
	let now = OffsetDateTime::now_utc().date();

	for (emote_id, count) in true_emote_stats {
		let current_count = emote_stats
			.range((emote_id, time::Date::MIN)..=(emote_id, time::Date::MAX))
			.map(|(_, count)| count)
			.sum::<i32>();

		let delta = count - current_count;
		if delta != 0 {
			*emote_stats.entry((emote_id, now)).or_default() += delta;
		}
	}

	let mut total = 0;

	for ((emote_id, date), count) in emote_stats {
		inserter.write(&EmoteStat { count, date, emote_id }).await.context("write")?;

		total += 1;
		if total > 10_000 {
			inserter.end().await.context("end")?;
			inserter = clickhouse.insert("emote_stats").context("insert")?.with_timeouts(None, None);
			total = 0;
		}
	}

	if total > 0 {
		inserter.end().await.context("end")?;
	}

	Ok(())
}
