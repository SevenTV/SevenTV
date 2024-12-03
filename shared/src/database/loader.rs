use std::collections::{HashMap, HashSet};
use std::future::IntoFuture;

use dataloader::BatchLoad;
use futures::{TryFutureExt, TryStreamExt};
use mongodb::options::ReadPreference;
use scuffle_batching::dataloader::DataLoader;
use scuffle_batching::DataLoaderFetcher;
use scuffle_metrics::metrics;
use serde::de::DeserializeOwned;

use super::MongoCollection;

pub struct LoaderById<T> {
	db: mongodb::Database,
	name: String,
	_phantom: std::marker::PhantomData<T>,
}

#[metrics]
pub mod dataloader {
	use scuffle_metrics::{HistogramF64, UpDownCounterI64};

	pub use super::*;

	fn inflight_keys(loader: &str) -> UpDownCounterI64;
	fn inflight_batches(loader: &str) -> UpDownCounterI64;

	#[builder = HistogramBuilder::default()]
	fn batch_load(loader: &str) -> HistogramF64;

	pub struct BatchLoad<'a>(&'a str, std::time::Instant, i64);

	impl<'a> BatchLoad<'a> {
		pub fn new(name: &'a str, key_count: usize) -> Self {
			inflight_keys(name).incr_by(key_count as i64);
			inflight_batches(name).incr();
			Self(name, std::time::Instant::now(), key_count as i64)
		}
	}

	impl Drop for BatchLoad<'_> {
		fn drop(&mut self) {
			inflight_keys(self.0).decr_by(self.2);
			inflight_batches(self.0).decr();
			batch_load(self.0).observe(self.1.elapsed().as_secs_f64());
		}
	}
}

impl<T: MongoCollection + DeserializeOwned + Clone + 'static> LoaderById<T> {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			format!("LoaderById<{}>", T::COLLECTION_NAME),
			1000,
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		db: mongodb::Database,
		name: String,
		batch_size: usize,
		concurrency: usize,
		delay: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(
			Self {
				db,
				name,
				_phantom: std::marker::PhantomData,
			},
			batch_size,
			concurrency,
			delay,
		)
	}
}

impl<T: MongoCollection + DeserializeOwned + Clone + 'static> DataLoaderFetcher for LoaderById<T> {
	type Key = T::Id;
	type Value = T;

	async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
		// We use an untyped find here because its not possible to do compile time macro
		// checks if the type is generic. This is a limitation of rust macros. However
		// this is entirely safe because every document is guaranteed to have an `_id`
		// field and that field is always going to have a `T::Id` type.
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<T> = T::collection(&self.db)
			.untyped()
			.find(bson::doc! {
				"_id": {
					"$in": bson::to_bson(&keys).unwrap(),
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

		let results: HashMap<_, _> = results.into_iter().map(|r| (r.id(), r)).collect();

		Some(results)
	}
}
