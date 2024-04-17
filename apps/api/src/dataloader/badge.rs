use futures::{TryFutureExt, TryStreamExt};
use mongodb::bson::oid::ObjectId;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{Badge, Collection};

pub struct BadgeByIdLoader {
	db: mongodb::Database,
}

impl BadgeByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for BadgeByIdLoader {
	type Error = ();
	type Key = ObjectId;
	type Value = Badge;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = Self::Value::collection(&self.db)
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
