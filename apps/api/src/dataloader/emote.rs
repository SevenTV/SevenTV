use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::emote::{Emote, EmoteId};
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct EmoteByUserIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl EmoteByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "EmoteByUserIdLoader".to_string(),
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

impl Loader for EmoteByUserIdLoader {
	type Key = UserId;
	type Value = Vec<Emote>;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<_> = Emote::collection(&self.db)
			.find(filter::filter! {
				Emote {
					#[query(selector = "in")]
					owner_id: keys,
					deleted: false,
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|e| e.owner_id))
	}
}

pub struct EmoteByIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl EmoteByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "EmoteByIdLoader".to_string(),
				concurrency: 50,
				max_batch_size: 1_000,
				sleep_duration: std::time::Duration::from_millis(5),
			},
		)
	}

	pub fn new_with_config(db: mongodb::Database, config: BatcherConfig) -> DataLoader<Self> {
		DataLoader::new(Self {
			db,
			config,
		})
	}
}

impl Loader for EmoteByIdLoader {
	type Key = EmoteId;
	type Value = Emote;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Vec<Emote> = Emote::collection(&self.db)
			.find(filter::filter! {
				Emote {
					#[query(rename = "_id", selector = "in")]
					id: keys,
					deleted: false,
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id(), r)).collect())
	}
}
