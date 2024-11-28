use std::sync::Arc;

use anyhow::Context;
use scuffle_context::ContextFutExt;
use shared::database::cron_job::{CronJob, CronJobId, CronJobInterval};
use shared::database::queries::{filter, update};
use shared::database::{Id, MongoCollection};
use tracing::Instrument;

use crate::global::Global;

mod emote_stats;
mod sub_refresh;

pub async fn run(global: Arc<Global>, ctx: scuffle_context::Context) {
	tracing::info!("started cron job runner");

	loop {
		if tokio::time::sleep(std::time::Duration::from_secs(5)).with_context(&ctx).await.is_none() {
			break;
		}

		let leased_id = Id::new();

		let job = match fetch_job(&global, leased_id).with_context(&ctx).await {
			Some(Ok(Some(job))) => job,
			Some(Ok(None)) => continue,
			Some(Err(e)) => {
				tracing::error!(error = %e, "failed to fetch job");
				continue;
			}
			None => break,
		};

		let span = tracing::info_span!("cron_job", job = %job.name, id = ?job.id);

		async {
			let job_id = job.id;

			tokio::select! {
				r = lease_job(&global, leased_id, job_id) => {
					if let Err(e) = r {
						tracing::error!(error = %e, "job failed");
					} else {
						tracing::info!("lost lock on job");
					}
				},
				r = run_job(&global, job, leased_id).with_context(&ctx) => {
					match r {
						Some(Ok(())) => {
							tracing::info!("job succeeded");
							return;
						},
						Some(Err(e)) => {
							tracing::error!("job failed: {:#}", e);
						}
						None => {
							tracing::info!("shutting down, cancelling job");
						}
					}
				}
			}

			if let Err(err) = free_job(&global, leased_id, job_id).await {
				tracing::error!("failed to free job: {:#}", err);
			}
		}
		.instrument(span)
		.await;
	}
}

async fn fetch_job(global: &Arc<Global>, id: Id) -> Result<Option<CronJob>, mongodb::error::Error> {
	let now = chrono::Utc::now();

	CronJob::collection(&global.db)
		.find_one_and_update(
			filter::filter! {
				CronJob {
					enabled: true,
					#[query(selector = "lt")]
					next_run: now,
					#[query(selector = "lt")]
					held_until: now,
				}
			},
			update::update! {
				#[query(set)]
				CronJob {
					held_until: now + chrono::Duration::minutes(1),
					updated_at: now,
					currently_running_by: id,
					search_updated_at: &None,
				}
			},
		)
		.await
}

async fn lease_job(global: &Arc<Global>, lease_id: Id, cron_job_id: CronJobId) -> Result<(), mongodb::error::Error> {
	loop {
		tokio::time::sleep(std::time::Duration::from_secs(30)).await;

		let now = chrono::Utc::now();

		if CronJob::collection(&global.db)
			.update_one(
				filter::filter! {
					CronJob {
						#[query(rename = "_id")]
						id: cron_job_id,
						currently_running_by: Some(lease_id),
						enabled: true,
					}
				},
				update::update! {
					#[query(set)]
					CronJob {
						held_until: now + chrono::Duration::minutes(1),
						updated_at: now,
						search_updated_at: &None,
					}
				},
			)
			.await?
			.modified_count
			!= 1
		{
			return Ok(());
		}

		tracing::info!("job leased again");
	}
}

async fn run_job(global: &Arc<Global>, job: CronJob, id: Id) -> anyhow::Result<()> {
	let job_id = job.id;
	let interval = job.interval;

	match job_id {
		CronJobId::SubscriptionRefresh => sub_refresh::run(global, job).await.context("sub refresh")?,
		CronJobId::EmoteScoresUpdate => emote_stats::run(global, job).await.context("emote stats")?,
	}

	complete_job(global, job_id, interval, id).await.context("complete job")?;

	Ok(())
}

async fn complete_job(
	global: &Arc<Global>,
	job_id: CronJobId,
	interval: CronJobInterval,
	currently_running_by: Id,
) -> Result<(), mongodb::error::Error> {
	let now = chrono::Utc::now();
	let next_run = now
		+ match interval {
			CronJobInterval::Hours(hours) => chrono::Duration::hours(hours as i64),
			CronJobInterval::Days(days) => chrono::Duration::days(days as i64),
		};

	tracing::info!("completing job");

	CronJob::collection(&global.db)
		.update_one(
			filter::filter! {
				CronJob {
					#[query(rename = "_id")]
					id: job_id,
					currently_running_by: Some(currently_running_by),
				}
			},
			update::update! {
				#[query(set)]
				CronJob {
					currently_running_by: &None,
					next_run,
					last_run: Some(now),
					held_until: now,
					updated_at: now,
					search_updated_at: &None,
				}
			},
		)
		.await?;

	Ok(())
}

async fn free_job(
	global: &Arc<Global>,
	currently_running_by: Id,
	cron_job_id: CronJobId,
) -> Result<(), mongodb::error::Error> {
	tracing::info!("freeing job");

	let now = chrono::Utc::now();

	CronJob::collection(&global.db)
		.find_one_and_update(
			filter::filter! {
				CronJob {
					id: cron_job_id,
					currently_running_by: Some(currently_running_by),
				}
			},
			update::update! {
				#[query(set)]
				CronJob {
					next_run: now + chrono::Duration::minutes(1),
					held_until: now,
					updated_at: now,
					search_updated_at: &None,
					currently_running_by: &None,
				}
			},
		)
		.await?;

	Ok(())
}
