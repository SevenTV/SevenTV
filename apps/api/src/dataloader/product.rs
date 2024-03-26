use std::sync::Arc;

use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use ulid::Ulid;

use crate::database::Product;

pub struct ProductByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl ProductByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for ProductByIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Product;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = scuffle_utils::database::query("SELECT * FROM products WHERE id = ANY($1)")
			.bind(keys)
			.build_query_as()
			.fetch_all(&self.db)
			.await
			.map_err(|e| {
				tracing::error!(err = %e, "failed to fetch categories by id");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}
