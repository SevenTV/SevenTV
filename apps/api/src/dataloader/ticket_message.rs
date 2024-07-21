use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use shared::database::queries::filter;
use shared::database::ticket::{TicketId, TicketMessage};
use shared::database::MongoCollection;

pub struct TicketMessageByTicketIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl TicketMessageByTicketIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "TicketMessageByTicketIdLoader".to_string(),
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

impl Loader for TicketMessageByTicketIdLoader {
	type Key = TicketId;
	type Value = Vec<TicketMessage>;

	fn config(&self) -> scuffle_foundations::batcher::BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Vec<_> = TicketMessage::collection(&self.db)
			.find(filter::filter! {
				TicketMessage {
					#[query(selector = "in")]
					ticket_id: keys,
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|e| e.ticket_id))
	}
}
