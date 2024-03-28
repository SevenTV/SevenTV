use std::sync::Arc;

use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use ulid::Ulid;

use crate::database::Badge;

pub struct BadgeLoader {
    db: Arc<scuffle_utils::database::Pool>,
}

impl BadgeLoader {
    pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
        DataLoader::new(Self { db })
    }
}

impl Loader for BadgeLoader {
    type Error = ();
	type Key = Ulid;
	type Value = Badge;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = scuffle_utils::database::query("SELECT * FROM badges WHERE id = ANY($1)")
			.bind(keys)
			.build_query_as()
			.fetch_all(&self.db)
			.await
			.map_err(|e| {
				tracing::error!(err = %e, "failed to fetch badges by id");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}
