use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use shared::database::loader::dataloader::BatchLoad;
use shared::database::queries::{filter, update};
use shared::database::user::session::{UserSession, UserSessionId};
use shared::database::MongoCollection;

pub struct UserSessionUpdaterBatcher {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl UserSessionUpdaterBatcher {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "UserSessionUpdaterBatcher".to_string(),
				concurrency: 500,
				max_batch_size: 1000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self { db, config })
	}
}

impl Loader for UserSessionUpdaterBatcher {
	type Key = UserSessionId;
	type Value = bool;

	fn config(&self) -> scuffle_foundations::batcher::BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len(), name = %self.config.name))]
	async fn fetch(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let _batch = BatchLoad::new(&self.config.name, keys.len());

		UserSession::collection(&self.db)
			.update_many(
				filter::filter! {
					UserSession {
						#[query(rename = "_id", selector = "in")]
						id: &keys,
					}
				},
				update::update! {
					#[query(set)]
					UserSession {
						last_used_at: chrono::Utc::now(),
					}
				},
			)
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(keys.into_iter().map(|k| (k, true)).collect())
	}
}
