use std::collections::HashMap;
use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::queries::filter;
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::User;
use shared::database::MongoCollection;

pub struct UserByPlatformIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl UserByPlatformIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "UserByPlatformIdLoader".to_string(),
				concurrency: 50,
				max_batch_size: 1_000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self { db, config })
	}
}

impl Loader for UserByPlatformIdLoader {
	type Key = (Platform, String);
	type Value = User;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let users: Vec<User> = User::collection(&self.db)
			.find(filter::Filter::or(keys.into_iter().map(|(platform, platform_id)| {
				filter::filter! {
					User {
						#[query(flatten)]
						connections: UserConnection {
							platform,
							platform_id,
						},
					}
				}
			})))
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		let mut results = HashMap::new();

		for user in users {
			for connection in &user.connections {
				results.insert((connection.platform, connection.platform_id.clone()), user.clone());
			}
		}

		Ok(results)
	}
}
