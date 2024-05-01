use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telementry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::{Collection, Paint, PaintId};

pub struct PaintByIdLoader {
	db: mongodb::Database,
}

impl PaintByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("PaintByIdLoader", Self { db })
	}
}

impl Loader for PaintByIdLoader {
	type Error = ();
	type Key = PaintId;
	type Value = Paint;

	#[tracing::instrument(name = "PaintByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Self::Value> = Paint::collection(&self.db)
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
