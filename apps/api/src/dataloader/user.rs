use std::collections::{HashMap, HashSet};
use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
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

		let grouped = keys.iter().map(|(k, v)| (k, v)).into_group_map();

		let users: Vec<User> = User::collection(&self.db)
			.find(filter::Filter::or(grouped.into_iter().map(|(platform, platform_ids)| {
				filter::filter! {
					User {
						#[query(elem_match)]
						connections: UserConnection {
							platform,
							#[query(selector = "in")]
							platform_id: platform_ids,
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

		let keys = keys.into_iter().collect::<HashSet<_>>();

		for user in users {
			for connection in &user.connections {
				let key = (connection.platform, connection.platform_id.clone());
				if !keys.contains(&key) {
					continue;
				}

				results.insert(key, user.clone());
			}
		}

		Ok(results)
	}
}

pub struct UserByPlatformUsernameLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl UserByPlatformUsernameLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "UserByPlatformUserameLoader".to_string(),
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

impl Loader for UserByPlatformUsernameLoader {
	type Key = (Platform, String);
	type Value = User;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let grouped = keys.iter().map(|(k, v)| (k, v)).into_group_map();

		let users: Vec<User> = User::collection(&self.db)
			.find(filter::Filter::or(grouped.into_iter().map(
				|(platform, platform_usernames)| {
					filter::filter! {
						User {
							#[query(elem_match)]
							connections: UserConnection {
								platform,
								#[query(selector = "in")]
								platform_username: platform_usernames,
							},
						}
					}
				},
			)))
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		let mut results = HashMap::new();
		let keys = keys.into_iter().collect::<HashSet<_>>();

		for user in users {
			for connection in &user.connections {
				let key = (connection.platform, connection.platform_username.clone());
				if !keys.contains(&key) {
					continue;
				}

				results.insert(key, user.clone());
			}
		}

		Ok(results)
	}
}
