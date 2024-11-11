use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::options::ReadPreference;
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use shared::database::loader::dataloader::BatchLoad;
use shared::database::product::subscription::{SubscriptionId, SubscriptionPeriod};
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct SubscriptionPeriodsByUserIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl SubscriptionPeriodsByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "SubscriptionPeriodsByUserIdLoader".to_string(),
				concurrency: 500,
				max_batch_size: 1000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self { db, config })
	}
}

impl Loader for SubscriptionPeriodsByUserIdLoader {
	type Key = UserId;
	type Value = Vec<SubscriptionPeriod>;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len(), name = %self.config.name))]
	async fn fetch(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let _batch = BatchLoad::new(&self.config.name, keys.len());

		let results: Vec<_> = SubscriptionPeriod::collection(&self.db)
			.find(filter::filter! {
				SubscriptionPeriod {
					#[query(flatten)]
					subscription_id: SubscriptionId {
						#[query(selector = "in")]
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
			})?;

		Ok(results.into_iter().into_group_map_by(|p| p.subscription_id.user_id))
	}
}

pub struct ActiveSubscriptionPeriodByUserIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl ActiveSubscriptionPeriodByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "ActiveSubscriptionPeriodByUserIdLoader".to_string(),
				concurrency: 500,
				max_batch_size: 1000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self { db, config })
	}
}

impl Loader for ActiveSubscriptionPeriodByUserIdLoader {
	type Key = UserId;
	type Value = SubscriptionPeriod;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len(), name = %self.config.name))]
	async fn fetch(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let _batch = BatchLoad::new(&self.config.name, keys.len());

		let results: Vec<_> = SubscriptionPeriod::collection(&self.db)
			.find(filter::Filter::and([
				filter::filter! {
					SubscriptionPeriod {
						#[query(flatten)]
						subscription_id: SubscriptionId {
							#[query(selector = "in")]
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
			})?;

		Ok(results.into_iter().map(|p| (p.subscription_id.user_id, p)).collect())
	}
}
