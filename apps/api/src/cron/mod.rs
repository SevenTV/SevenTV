use std::sync::Arc;

use anyhow::Context;
use shared::database::{cron_job::{CronJob, CronJobId, CronJobInterval}, queries::{filter, update}, Id, MongoCollection};

use crate::global::Global;

mod sub_refresh;
mod emote_stats;

pub async fn run(global: Arc<Global>) {
    let id = Id::<()>::new();

	loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;

        let job = match fetch_job(&global, id.clone()).await {
            Ok(Some(job)) => job,
            Ok(None) => continue,
            Err(e) => {
                tracing::error!(error = %e, "failed to fetch job");
                continue;
            }
        };

        let span = tracing::info_span!("cron_job", job = %job.name, id = ?job.id);
        let _enter = span.enter();

        let job_id = job.id;

        tokio::select! {
            r = refresh_job(&global, id.clone(), job_id) => {
                if let Err(e) = r {
                    tracing::error!(error = %e, "job failed");
                } else {
                    tracing::info!("lost lock on job");
                }
            },
            r = run_job(&global, job, id) => {
                if let Err(e) = r {
                    tracing::error!(error = %e, "job failed");
                } else {
                    tracing::info!("job succeeded");
                    continue;
                }
            }
        }

        if let Err(err) = free_job(&global, id, job_id).await {
            tracing::error!(error = %err, "failed to free job");
        }
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
                    currently_running_by: id,
                }
            },
        )
        .await
}

async fn refresh_job(global: &Arc<Global>, _id: Id, cron_job_id: CronJobId) -> Result<(), mongodb::error::Error> {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        if CronJob::collection(&global.db)
            .update_one(
                filter::filter! {
                    CronJob {
                        id: cron_job_id,
                        currently_running_by: Some(_id),
                        enabled: true,
                    }
                },
                update::update! {
                    #[query(set)]
                    CronJob {
                        held_until: chrono::Utc::now() + chrono::Duration::minutes(1),
                        updated_at: chrono::Utc::now(),
                    }
                },
            )
            .await?
            .modified_count != 1 {
                return Ok(());
            }
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

async fn complete_job(global: &Arc<Global>, job_id: CronJobId, interval: CronJobInterval, _id: Id) -> Result<(), mongodb::error::Error> {
    let next_run = chrono::Utc::now() + match interval {
        CronJobInterval::Hours(hours) => chrono::Duration::hours(hours as i64),
        CronJobInterval::Days(days) => chrono::Duration::days(days as i64),
    };

    CronJob::collection(&global.db)
        .update_one(
            filter::filter! {
                CronJob {
                    id: job_id,
                    currently_running_by: Some(_id),
                }
            },
            update::update! {
                #[query(unset)]
                CronJob {
                    currently_running_by: true,
                },
                #[query(set)]
                CronJob {
                    next_run,
                    last_run: Some(chrono::Utc::now()),
                    held_until: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            },
        )
        .await?;

    Ok(())
}

async fn free_job(global: &Arc<Global>, _id: Id, cron_job_id: CronJobId) -> Result<(), mongodb::error::Error> {
    CronJob::collection(&global.db)
        .find_one_and_update(
            filter::filter! {
                CronJob {
                    id: cron_job_id,
                    currently_running_by: Some(_id),
                }
            },
            update::update! {
                #[query(unset)]
                CronJob {
                    currently_running_by: true,
                },
                #[query(set)]
                CronJob {
                    next_run: chrono::Utc::now(),
                    held_until: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            },
        )
        .await?;

    Ok(())
}
