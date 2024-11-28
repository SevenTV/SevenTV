use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::options::ReadPreference;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::queries::filter;
use shared::database::ticket::{TicketId, TicketMessage};
use shared::database::MongoCollection;

pub struct TicketMessageByTicketIdLoader {
	db: mongodb::Database,
	name: String,
}

impl TicketMessageByTicketIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"TicketMessageByTicketIdLoader".to_string(),
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		db: mongodb::Database,
		name: String,
		batch_size: usize,
		sleep_duration: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { db, name }, batch_size, sleep_duration)
	}
}

impl DataLoaderFetcher for TicketMessageByTicketIdLoader {
	type Key = TicketId;
	type Value = Vec<TicketMessage>;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<_> = TicketMessage::collection(&self.db)
			.find(filter::filter! {
				TicketMessage {
					#[query(selector = "in", serde)]
					ticket_id: keys,
				}
			})
			.batch_size(1000)
			.selection_criteria(ReadPreference::SecondaryPreferred { options: None }.into())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})
			.ok()?;

		Some(results.into_iter().into_group_map_by(|e| e.ticket_id))
	}
}
