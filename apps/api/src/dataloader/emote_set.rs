use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::options::ReadPreference;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::emote_set::EmoteSet;
use shared::database::loader::dataloader::BatchLoad;
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct EmoteSetByUserIdLoader {
	name: String,
	db: mongodb::Database,
}

impl EmoteSetByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"EmoteSetByUserIdLoader".to_string(),
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

impl DataLoaderFetcher for EmoteSetByUserIdLoader {
	type Key = UserId;
	type Value = Vec<EmoteSet>;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<EmoteSet> = EmoteSet::collection(&self.db)
			.find(filter::filter! {
				EmoteSet {
					#[query(selector = "in", serde)]
					owner_id: keys,
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

		Some(results.into_iter().into_group_map_by(|r| r.owner_id.unwrap()))
	}
}
