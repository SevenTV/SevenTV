use std::collections::HashSet;
use std::sync::Arc;

use futures::StreamExt;
use shared::database::cron_job::CronJob;
use shared::database::product::subscription::SubscriptionPeriod;
use shared::database::queries::filter;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::sub_refresh_job::refresh;

pub async fn run(global: &Arc<Global>, _job: CronJob) -> anyhow::Result<()> {
	tracing::info!("started subscription refresh job");

	let mut cursor = SubscriptionPeriod::collection(&global.db)
		.find(filter::filter! {
			SubscriptionPeriod {}
		})
		.await?;

	let mut subs = HashSet::new();

	while let Some(period) = cursor.next().await.transpose()? {
		subs.insert(period.subscription_id);
	}

	tracing::info!("found {} subscriptions", subs.len());

	// Do updates in batches of 1000
	let total = subs.len();

	let semaphore = &tokio::sync::Semaphore::new(1000);
	let mut futures = futures::stream::FuturesUnordered::from_iter(subs.into_iter().map(|sub| async move {
		let _ticket = semaphore.acquire().await.unwrap();
		refresh(global, sub).await.map_err(|err| (sub, err)).map(|_| sub)
	}));

	let mut error_count = 0;

	while let Some(sub) = futures.next().await {
		match sub {
			Ok(sub) => {
				tracing::debug!(subscription = %sub, "refreshed subscription");
			}
			Err((sub, err)) => {
				tracing::error!(subscription = %sub, error = ?err, "failed to refresh subscription");
				error_count += 1;
				if error_count > total / 10 {
					anyhow::bail!("too many errors");
				}
			}
		}
	}

	Ok(())
}
