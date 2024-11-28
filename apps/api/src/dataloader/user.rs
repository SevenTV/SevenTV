use std::collections::{HashMap, HashSet};
use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::queries::filter;
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::User;
use shared::database::MongoCollection;

pub struct UserByPlatformIdLoader {
	db: mongodb::Database,
	name: String,
}

impl UserByPlatformIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"UserByPlatformIdLoader".to_string(),
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(db: mongodb::Database, name: String, batch_size: usize, sleep_duration: std::time::Duration) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, sleep_duration)
	}
}

impl DataLoaderFetcher for UserByPlatformIdLoader {
	type Key = (Platform, String);
	type Value = User;

	async fn load(&self, keys: std::collections::HashSet<Self::Key>) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

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
			})
			.ok()?;

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

		Some(results)
	}
}

pub struct UserByPlatformUsernameLoader {
	db: mongodb::Database,
	name: String,
}

impl UserByPlatformUsernameLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"UserByPlatformUserameLoader".to_string(),
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(db: mongodb::Database, name: String, batch_size: usize, sleep_duration: std::time::Duration) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, sleep_duration)
	}
}

impl DataLoaderFetcher for UserByPlatformUsernameLoader {
	type Key = (Platform, String);
	type Value = User;

	async fn load(&self, keys: std::collections::HashSet<Self::Key>) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

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
			})
			.ok()?;

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

		Some(results)
	}
}
