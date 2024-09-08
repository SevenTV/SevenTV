use std::sync::Arc;

use shared::database::{cron_job, MongoCollection};

use super::{Job, ProcessOutcome};
use crate::global::Global;

pub struct CronJobsJob {
	global: Arc<Global>,
}

impl Job for CronJobsJob {
	type T = cron_job::CronJob;

	const NAME: &'static str = "cron_jobs";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping cron_jobs collections");
			cron_job::CronJob::collection(global.target_db()).untyped().drop().await?;
			let indexes = cron_job::CronJob::indexes();
			if !indexes.is_empty() {
				cron_job::CronJob::collection(global.target_db())
					.untyped()
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self { global })
	}

	async fn finish(self) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		if let Err(err) = cron_job::CronJob::collection(self.global.target_db())
			.insert_many(cron_job::default_cron_jobs())
			.await
		{
			outcome = outcome.with_error(err);
		}

		outcome
	}
}
