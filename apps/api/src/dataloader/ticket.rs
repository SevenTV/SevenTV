use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{Collection, Ticket, TicketId, TicketMember, TicketMessage};

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

pub struct TicketMembersByTicketIdLoader {
    pub db: mongodb::Database,
}

impl TicketMembersByTicketIdLoader {
    pub fn new(db: mongodb::Database) -> DataLoader<Self> {
        DataLoader::new("TicketMembersByTicketIdLoader", Self { db })
    }
}

impl Loader for TicketMembersByTicketIdLoader {
    type Error = ();
    type Key = TicketId;
    type Value = Vec<TicketMember>;

    #[tracing::instrument(name = "TicketMembersByTicketIdLoader::load", skip(self), fields(key_count = keys.len()))]
    async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
        let results: Vec<TicketMember> = TicketMember::collection(&self.db)
            .find(
                mongodb::bson::doc! {
                    "ticket_id": {
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

        Ok(results.into_iter().into_group_map_by(|m| m.ticket_id))
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
            .find(
                mongodb::bson::doc! {
                    "ticket_id": {
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

        Ok(results.into_iter().into_group_map_by(|m| m.ticket_id))
    }
}