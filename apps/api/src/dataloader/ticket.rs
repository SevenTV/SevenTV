use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::ticket::{Ticket, TicketId};
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
			.find(
				mongodb::bson::doc! {
					"_id": {
						"$in": keys,
					}
				},
				None,
			)
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}
