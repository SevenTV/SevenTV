use futures::{TryFutureExt, TryStreamExt};
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{Collection, FileSet, FileSetId};

pub struct FileSetByIdLoader {
	db: mongodb::Database,
}

impl FileSetByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for FileSetByIdLoader {
	type Error = ();
	type Key = FileSetId;
	type Value = FileSet;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = FileSet::collection(&self.db)
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
