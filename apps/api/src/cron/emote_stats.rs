use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use anyhow::Context;
use fred::prelude::KeysInterface;
use shared::database::cron_job::CronJob;
use shared::database::emote::{Emote, EmoteId, EmoteScores};
use shared::database::queries::{filter, update};
use shared::database::updater::MongoReq;
use time::OffsetDateTime;

use crate::global::Global;

#[derive(Debug, serde::Deserialize, serde::Serialize, clickhouse::Row)]
pub struct EmoteStat {
	pub count: i32,
	pub emote_id: EmoteId,
}

async fn fetch(
	cursor: Result<clickhouse::query::RowCursor<EmoteStat>, clickhouse::error::Error>,
	mut cb: impl FnMut(EmoteStat),
) -> anyhow::Result<()> {
	let mut cursor = cursor?;

	while let Some(stat) = cursor.next().await? {
		cb(stat);
	}

	Ok(())
}

pub async fn run(global: &Arc<Global>, _job: CronJob) -> anyhow::Result<()> {
	tracing::info!("started emote stats job");

	let mut scores = HashMap::<EmoteId, EmoteScores>::new();

	let mut top_daily = BTreeSet::<(i32, EmoteId)>::new();
	let mut top_weekly = BTreeSet::<(i32, EmoteId)>::new();
	let mut top_monthly = BTreeSet::<(i32, EmoteId)>::new();
	let mut top_all_time = BTreeSet::<(i32, EmoteId)>::new();
	let mut trending_day = BTreeSet::<(i32, EmoteId)>::new();
	let mut trending_week = BTreeSet::<(i32, EmoteId)>::new();
	let mut trending_month = BTreeSet::<(i32, EmoteId)>::new();

	macro_rules! update_stat {
		($stats:expr, $stat:ident, $count:ident, $n:expr) => {
			$stats.entry($stat.emote_id).or_default().$count = $stat.count as i32;
			$count.insert(($stat.count as i32, $stat.emote_id));

			if $count.len() > $n {
				// removes the smallest element
				$count.pop_first();
			}
		};
	}

	let global_config = global
		.global_config_loader
		.load(())
		.await
		.map_err(|_| anyhow::anyhow!("failed to load global config"))?
		.context("failed to load global config")?;

	let now = OffsetDateTime::now_utc().date();

	let mut total = 0;
	fetch(
		global
			.clickhouse
			.query("SELECT sum(count) count, emote_id FROM emote_stats WHERE date >= ? GROUP BY emote_id")
			.bind((now - time::Duration::days(2)).to_string())
			.fetch(),
		|stat| {
			update_stat!(scores, stat, top_daily, global_config.trending_emote_count);
			update_stat!(scores, stat, trending_day, global_config.trending_emote_count);
			total += 1;
		},
	)
	.await
	.context("last day")?;

	tracing::info!("fetched {total} entries for last day");

	total = 0;
	fetch(
		global
			.clickhouse
			.query("SELECT sum(count) count, emote_id FROM emote_stats WHERE date >= ? GROUP BY emote_id")
			.bind((now - time::Duration::weeks(1)).to_string())
			.fetch(),
		|stat| {
			update_stat!(scores, stat, top_weekly, global_config.trending_emote_count);
			update_stat!(scores, stat, trending_week, global_config.trending_emote_count);
			total += 1;
		},
	)
	.await
	.context("last week")?;

	tracing::info!("fetched {total} entries for last week");

	total = 0;
	fetch(
		global
			.clickhouse
			.query("SELECT sum(count) count, emote_id FROM emote_stats WHERE date >= ? GROUP BY emote_id")
			.bind((now - time::Duration::days(30)).to_string())
			.fetch(),
		|stat| {
			update_stat!(scores, stat, top_monthly, global_config.trending_emote_count);
			update_stat!(scores, stat, trending_month, global_config.trending_emote_count);
			total += 1;
		},
	)
	.await
	.context("last month")?;

	tracing::info!("fetched {total} entries for last month");

	total = 0;
	fetch(
		global
			.clickhouse
			.query("SELECT sum(count) count, emote_id FROM emote_stats GROUP BY emote_id")
			.fetch(),
		|stat| {
			update_stat!(scores, stat, top_all_time, global_config.trending_emote_count);
			total += 1;
		},
	)
	.await
	.context("all time")?;

	tracing::info!("fetched {total} entries for all time");

	tracing::info!("found {} entries", scores.len());

	let scores = scores.into_iter().collect::<Vec<_>>();
	let chunks = scores.chunks(10000);

	let now = chrono::Utc::now();

	for chunk in chunks {
		global
			.updater
			.bulk(chunk.iter().map(|(id, scores)| {
				MongoReq::update(
					filter::filter! {
						Emote {
							#[query(rename = "_id")]
							id,
						}
					},
					update::update! {
						#[query(set)]
						Emote {
							#[query(serde)]
							scores,
							updated_at: now,
						}
					},
					false,
				)
			}))
			.await
			.into_iter()
			.collect::<Result<Vec<_>, _>>()
			.context("update scores")?;
	}

	macro_rules! update_redis {
		($global:ident, [$($field:ident),+,],) => {
			$(
				$global
					.redis
					.set::<(), _, _>(
						format!("emote_stats:{}", stringify!($field)),
						serde_json::to_string(&$field
							.into_iter()
							.rev()
							.map(|(_, id)| id)
							.collect::<Vec<EmoteId>>()
						).unwrap(),
						None,
						None,
						false,
					)
					.await?;
			)+
		};
	}

	update_redis!(
		global,
		[
			top_daily,
			top_weekly,
			top_monthly,
			top_all_time,
			trending_day,
			trending_week,
			trending_month,
		],
	);

	Ok(())
}
