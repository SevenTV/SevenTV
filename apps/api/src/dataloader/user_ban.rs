use std::future::IntoFuture;

use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::options::ReadPreference;
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use shared::database::queries::filter;
use shared::database::user::ban::UserBan;
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct UserBanByUserIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl UserBanByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "UserBanByUserIdLoader".to_string(),
				concurrency: 50,
				max_batch_size: 1_000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self { db, config })
	}
}

impl Loader for UserBanByUserIdLoader {
	type Key = UserId;
	type Value = Vec<UserBan>;

	fn config(&self) -> scuffle_foundations::batcher::BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Vec<_> = UserBan::collection(&self.db)
			.find(filter::filter! {
				UserBan {
					#[query(selector = "in")]
					user_id: keys,
				}
			})
			.selection_criteria(ReadPreference::SecondaryPreferred { options: None }.into())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|e| e.user_id))
	}
}
