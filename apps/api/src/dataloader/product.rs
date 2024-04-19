use futures::{TryFutureExt, TryStreamExt};
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{Collection, Product, ProductId};

pub struct ProductByIdLoader {
	db: mongodb::Database,
}

impl ProductByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for ProductByIdLoader {
	type Error = ();
	type Key = ProductId;
	type Value = Product;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = Product::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"_id": {
						"$in": keys,
					}
				},
				None,
			)
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}
