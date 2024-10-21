use std::collections::HashMap;
use std::future::IntoFuture;

use dataloader::BatchLoad;
use futures::{TryFutureExt, TryStreamExt};
use mongodb::options::ReadPreference;
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use scuffle_foundations::telemetry::metrics::metrics;
use serde::de::DeserializeOwned;

use super::MongoCollection;

pub struct LoaderById<T> {
	db: mongodb::Database,
	config: BatcherConfig,
	_phantom: std::marker::PhantomData<T>,
}

#[metrics]
pub mod dataloader {
	use scuffle_foundations::telemetry::metrics::HistogramBuilder;

	pub use super::*;

	fn inflight_keys(loader: &str) -> prometheus_client::metrics::gauge::Gauge;
	fn inflight_batches(loader: &str) -> prometheus_client::metrics::gauge::Gauge;

	#[builder = HistogramBuilder::default()]
	fn batch_load(loader: &str) -> prometheus_client::metrics::histogram::Histogram;

	pub struct BatchLoad<'a>(&'a str, std::time::Instant, i64);

	impl<'a> BatchLoad<'a> {
		pub fn new(name: &'a str, key_count: usize) -> Self {
			inflight_keys(name).inc_by(key_count as i64);
			inflight_batches(name).inc();
			Self(name, std::time::Instant::now(), key_count as i64)
		}
	}

	impl Drop for BatchLoad<'_> {
		fn drop(&mut self) {
			inflight_keys(self.0).dec_by(self.2);
			inflight_batches(self.0).dec();
			batch_load(self.0).observe(self.1.elapsed().as_secs_f64());
		}
	}
}

impl<T: MongoCollection + DeserializeOwned + Clone + 'static> LoaderById<T> {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: format!("LoaderById<{}>", T::COLLECTION_NAME),
				concurrency: 500,
				max_batch_size: 1000,
				sleep_duration: std::time::Duration::from_millis(20),
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
		let _batch = BatchLoad::new(&self.config.name, keys.len());

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
			})?;

		let results: HashMap<_, _> = results.into_iter().map(|r| (r.id(), r)).collect();

		Ok(results)
	}
}
