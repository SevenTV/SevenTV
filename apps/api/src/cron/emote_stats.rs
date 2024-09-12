use std::sync::Arc;

use crate::global::Global;

use shared::database::cron_job::CronJob;

pub async fn run(_global: &Arc<Global>, _job: CronJob) -> anyhow::Result<()> {
    Ok(())
}
