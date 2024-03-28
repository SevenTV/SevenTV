use std::sync::Arc;

use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use ulid::Ulid;

use crate::database::File;

pub struct FileLoader {
    db: Arc<scuffle_utils::database::Pool>,
}

impl FileLoader {
    pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
        DataLoader::new(Self { db })
    }
}

impl Loader for FileLoader {
    type Error = ();
	type Key = Ulid;
	type Value = File;

    async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
        let results: Vec<Self::Value> = scuffle_utils::database::query("SELECT * FROM files WHERE id = ANY($1)")
			.bind(keys)
			.build_query_as()
			.fetch_all(&self.db)
			.await
			.map_err(|e| {
				tracing::error!(err = %e, "failed to fetch files by id");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
    }
}
