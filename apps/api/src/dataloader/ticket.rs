use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::ticket::{Ticket, TicketId, TicketMessage};
use shared::database::Collection;

pub struct TicketByIdLoader {
	pub db: mongodb::Database,
}

impl TicketByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("TicketByIdLoader", Self { db })
	}
}

impl Loader for TicketByIdLoader {
	type Error = ();
	type Key = TicketId;
	type Value = Ticket;

	#[tracing::instrument(name = "TicketByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = Ticket::collection(&self.db)
			.find(doc! {
				"_id": {
					"$in": keys,
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}

pub struct TicketMessagesByTicketIdLoader {
	pub db: mongodb::Database,
}

impl TicketMessagesByTicketIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("TicketMessagesByTicketIdLoader", Self { db })
	}
}

impl Loader for TicketMessagesByTicketIdLoader {
	type Error = ();
	type Key = TicketId;
	type Value = Vec<TicketMessage>;

	#[tracing::instrument(name = "TicketMessagesByTicketIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Vec<TicketMessage> = TicketMessage::collection(&self.db)
			.find(doc! {
				"ticket_id": {
					"$in": keys,
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|m| m.ticket_id))
	}
}
