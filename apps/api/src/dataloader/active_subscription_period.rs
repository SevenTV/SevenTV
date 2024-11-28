use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::options::ReadPreference;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::product::subscription::{SubscriptionId, SubscriptionPeriod};
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct SubscriptionPeriodsByUserIdLoader {
	db: mongodb::Database,
	name: String,
}

impl SubscriptionPeriodsByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"SubscriptionPeriodsByUserIdLoader".to_string(),
			500,
			50,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		db: mongodb::Database,
		name: String,
		batch_size: usize,
		concurrency: usize,
		sleep_duration: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, concurrency, sleep_duration)
	}
}

impl DataLoaderFetcher for SubscriptionPeriodsByUserIdLoader {
	type Key = UserId;
	type Value = Vec<SubscriptionPeriod>;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<_> = SubscriptionPeriod::collection(&self.db)
			.find(filter::filter! {
				SubscriptionPeriod {
					#[query(flatten)]
					subscription_id: SubscriptionId {
						#[query(selector = "in", serde)]
						user_id: &keys,
					},
				}
			})
			.batch_size(1000)
			.selection_criteria(ReadPreference::SecondaryPreferred { options: None }.into())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})
			.ok()?;

		Some(results.into_iter().into_group_map_by(|p| p.subscription_id.user_id))
	}
}

pub struct ActiveSubscriptionPeriodByUserIdLoader {
	db: mongodb::Database,
	name: String,
}

impl ActiveSubscriptionPeriodByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"ActiveSubscriptionPeriodByUserIdLoader".to_string(),
			500,
			50,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		db: mongodb::Database,
		name: String,
		batch_size: usize,
		concurrency: usize,
		sleep_duration: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, concurrency, sleep_duration)
	}
}

impl DataLoaderFetcher for ActiveSubscriptionPeriodByUserIdLoader {
	type Key = UserId;
	type Value = SubscriptionPeriod;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<_> = SubscriptionPeriod::collection(&self.db)
			.find(filter::Filter::and([
				filter::filter! {
					SubscriptionPeriod {
						#[query(flatten)]
						subscription_id: SubscriptionId {
							#[query(selector = "in", serde)]
							user_id: keys,
						},
						#[query(selector = "lt")]
						start: chrono::Utc::now(),
					}
				}
				.into(),
				filter::Filter::or([
					filter::filter! {
						SubscriptionPeriod {
							#[query(selector = "gt")]
							end: chrono::Utc::now(),
						}
					},
					filter::filter! {
						SubscriptionPeriod {
							#[query(selector = "gt")]
							end: chrono::Utc::now() + chrono::Duration::days(2),
							auto_renew: true,
						}
					},
				]),
			]))
			.batch_size(1000)
			.selection_criteria(ReadPreference::SecondaryPreferred { options: None }.into())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})
			.ok()?;

		Some(results.into_iter().map(|p| (p.subscription_id.user_id, p)).collect())
	}
}
