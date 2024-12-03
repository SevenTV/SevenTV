use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::queries::{filter, update};
use shared::database::user::session::{UserSession, UserSessionId};
use shared::database::MongoCollection;

pub struct UserSessionUpdaterBatcher {
	db: mongodb::Database,
	name: String,
}

impl UserSessionUpdaterBatcher {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"UserSessionUpdaterBatcher".to_string(),
			1000,
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		db: mongodb::Database,
		name: String,
		batch_size: usize,
		concurrency: usize,
		sleep_duration: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, concurrency, sleep_duration)
	}
}

impl DataLoaderFetcher for UserSessionUpdaterBatcher {
	type Key = UserSessionId;
	type Value = bool;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		UserSession::collection(&self.db)
			.update_many(
				filter::filter! {
					UserSession {
						#[query(rename = "_id", selector = "in", serde)]
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
			})
			.ok()?;

		Some(keys.into_iter().map(|k| (k, true)).collect())
	}
}
