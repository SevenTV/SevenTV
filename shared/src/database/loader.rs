use std::future::IntoFuture;

use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use serde::de::DeserializeOwned;

use super::MongoCollection;

pub struct LoaderById<T: MongoCollection + DeserializeOwned + Clone + 'static> {
	db: mongodb::Database,
	config: BatcherConfig,
	_phantom: std::marker::PhantomData<T>,
}

impl<T: MongoCollection + DeserializeOwned + Clone + 'static> LoaderById<T> {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: format!("LoaderById<{}>", T::COLLECTION_NAME),
				concurrency: 50,
				max_batch_size: 1_000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self {
			db,
			config,
			_phantom: std::marker::PhantomData,
		})
	}
}

impl<T: MongoCollection + DeserializeOwned + Clone + 'static> Loader for LoaderById<T> {
	type Key = T::Id;
	type Value = T;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		// We use an untyped find here because its not possible to do compile time macro
		// checks if the type is generic. This is a limitation of rust macros. However
		// this is entirely safe because every document is guaranteed to have an `_id`
		// field and that field is always going to have a `T::Id` type.
		let results: Vec<T> = T::collection(&self.db)
			.untyped()
			.find(bson::doc! {
				"_id": {
					"$in": bson::to_bson(&keys).unwrap(),
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id(), r)).collect())
	}
}
