use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use shared::database::cron_job::CronJob;
use shared::database::emote::{Emote, EmoteId, EmoteScores};
use shared::database::queries::{filter, update};
use shared::database::updater::MongoReq;
use time::OffsetDateTime;

use crate::global::Global;

#[derive(Debug, serde::Deserialize, clickhouse::Row)]
struct EmoteStat {
	pub count: usize,
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

	let now = OffsetDateTime::now_utc().date();

	let mut total = 0;
	fetch(
		global
			.clickhouse
			.query("SELECT sum(count) count, emote_id FROM emote_stats WHERE date >= ? GROUP BY emote_id")
			.bind((now - time::Duration::days(2)).to_string())
			.fetch(),
		|EmoteStat { emote_id, count }| {
			scores.entry(emote_id).or_default().top_daily = count as i32;
			scores.entry(emote_id).or_default().trending_day = count as i32;
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
		|EmoteStat { emote_id, count }| {
			scores.entry(emote_id).or_default().top_weekly = count as i32;
			scores.entry(emote_id).or_default().trending_week = count as i32;
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
		|EmoteStat { emote_id, count }| {
			scores.entry(emote_id).or_default().top_monthly = count as i32;
			scores.entry(emote_id).or_default().trending_month = count as i32;
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
		|EmoteStat { emote_id, count }| {
			scores.entry(emote_id).or_default().top_all_time = count as i32;
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

	Ok(())
}
