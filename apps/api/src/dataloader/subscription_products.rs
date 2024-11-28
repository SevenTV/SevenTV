use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use mongodb::options::ReadPreference;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::product::SubscriptionProduct;
use shared::database::queries::filter;
use shared::database::MongoCollection;

pub struct SubscriptionProductsLoader {
	db: mongodb::Database,
	name: String,
}

impl SubscriptionProductsLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"SubscriptionProductsLoader".to_string(),
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(db: mongodb::Database, name: String, batch_size: usize, sleep_duration: std::time::Duration) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, sleep_duration)
	}
}

impl DataLoaderFetcher for SubscriptionProductsLoader {
	type Key = ();
	type Value = Vec<SubscriptionProduct>;

	async fn load(&self, keys: std::collections::HashSet<Self::Key>) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Self::Value = SubscriptionProduct::collection(&self.db)
			.find(filter::filter! {
				SubscriptionProduct {}
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

		Some(std::iter::once(((), results)).collect())
	}
}
