use std::future::IntoFuture;

use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::options::ReadPreference;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::queries::filter;
use shared::database::user::ban::UserBan;
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct UserBanByUserIdLoader {
	db: mongodb::Database,
	name: String,
}

impl UserBanByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"UserBanByUserIdLoader".to_string(),
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(db: mongodb::Database, name: String, batch_size: usize, sleep_duration: std::time::Duration) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, sleep_duration)
	}
}

impl DataLoaderFetcher for UserBanByUserIdLoader {
	type Key = UserId;
	type Value = Vec<UserBan>;

	async fn load(&self, keys: std::collections::HashSet<Self::Key>) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<_> = UserBan::collection(&self.db)
			.find(filter::filter! {
				UserBan {
					#[query(selector = "in", serde)]
					user_id: keys,
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

		Some(results.into_iter().into_group_map_by(|e| e.user_id))
	}
}
