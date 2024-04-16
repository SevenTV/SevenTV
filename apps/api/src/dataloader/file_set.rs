use std::sync::Arc;

use mongodb::bson::oid::ObjectId;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::FileSet;

pub struct FileSetByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl FileSetByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for FileSetByIdLoader {
	type Error = ();
	type Key = ObjectId;
	type Value = FileSet;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = scuffle_utils::database::query("SELECT * FROM file_sets WHERE id = ANY($1)")
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
