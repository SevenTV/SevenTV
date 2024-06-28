use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::product::{Product, ProductId};
use shared::database::Collection;

pub struct ProductByIdLoader {
	db: mongodb::Database,
}

impl ProductByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("ProductByIdLoader", Self { db })
	}
}

impl Loader for ProductByIdLoader {
	type Error = ();
	type Key = ProductId;
	type Value = Product;

	#[tracing::instrument(name = "ProductByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let keys = keys.into_iter().map(|k| k.to_string()).collect::<Vec<_>>();

		let results: Vec<Self::Value> = Product::collection(&self.db)
			.find(doc! {
				"_id": {
					"$in": keys,
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id.clone(), r)).collect())
	}
}
