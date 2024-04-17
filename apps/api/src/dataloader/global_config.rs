use mongodb::bson::oid::ObjectId;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{Collection, GlobalConfig};

pub struct GlobalConfigLoader {
	db: mongodb::Database,
}

impl GlobalConfigLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for GlobalConfigLoader {
	type Error = ();
	type Key = ();
	type Value = GlobalConfig;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let config: GlobalConfig = Self::Value::collection(&self.db)
			.find_one(
				mongodb::bson::doc! {
					"_id": ObjectId::from_bytes(Default::default())
				},
				None,
			)
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?
			.unwrap_or_default();

		Ok([((), config)].into_iter().collect())
	}
}
