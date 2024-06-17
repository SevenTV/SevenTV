use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::global::GlobalConfig;
use shared::database::Collection;

pub struct GlobalConfigLoader {
	db: mongodb::Database,
}

impl GlobalConfigLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("GlobalConfigLoader", Self { db })
	}
}

impl Loader for GlobalConfigLoader {
	type Error = ();
	type Key = ();
	type Value = GlobalConfig;

	#[tracing::instrument(name = "GlobalConfigLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let config: GlobalConfig = GlobalConfig::collection(&self.db)
			.find_one(None, None)
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?
			.ok_or_else(|| {
				tracing::error!("failed to load: not found");
			})?;

		Ok([((), config)].into_iter().collect())
	}
}
