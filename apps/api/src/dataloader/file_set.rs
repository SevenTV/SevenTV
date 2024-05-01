use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telementry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::{Collection, FileSet, FileSetId};

pub struct FileSetByIdLoader {
	db: mongodb::Database,
}

impl FileSetByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("FileSetByIdLoader", Self { db })
	}
}

impl Loader for FileSetByIdLoader {
	type Error = ();
	type Key = FileSetId;
	type Value = FileSet;

	#[tracing::instrument(name = "FileSetByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

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
