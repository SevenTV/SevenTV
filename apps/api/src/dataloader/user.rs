use std::collections::HashMap;
use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use mongodb::options::FindOptions;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::user::connection::Platform;
use shared::database::user::{User, UserId};
use shared::database::Collection;

pub struct UserByIdLoader {
	db: mongodb::Database,
}

impl UserByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserByIdLoader", Self { db })
	}
}

impl Loader for UserByIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = User;

	#[tracing::instrument(name = "UserByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<User> = User::collection(&self.db)
			.find(doc! {
				"_id": {
					"$in": keys,
				}
			})
			.with_options(
				FindOptions::builder()
					.projection(doc! {
						"search_index.emote_ids": 0,
					})
					.build(),
			)
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}

pub struct UserByPlatformIdLoader {
	db: mongodb::Database,
}

impl UserByPlatformIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserByPlatformIdLoader", Self { db })
	}
}

impl Loader for UserByPlatformIdLoader {
	type Error = ();
	type Key = (Platform, String);
	type Value = User;

	#[tracing::instrument(name = "UserByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let users: Vec<User> = User::collection(&self.db)
			.find(doc! {
				"$or": keys.into_iter().map(|(platform, id)| {
					doc! {
						"connections.platform": platform as i32,
						"connections.platform_id": id,
					}
				}).collect::<Vec<_>>(),
			})
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
