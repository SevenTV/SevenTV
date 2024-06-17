use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::emote_set::{EmoteSet, EmoteSetId};
use shared::database::user::UserId;
use shared::database::Collection;

pub struct EmoteSetByIdLoader {
	db: mongodb::Database,
}

impl EmoteSetByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("EmoteSetByIdLoader", Self { db })
	}
}

impl Loader for EmoteSetByIdLoader {
	type Error = ();
	type Key = EmoteSetId;
	type Value = EmoteSet;

	#[tracing::instrument(name = "EmoteSetByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Self::Value> = EmoteSet::collection(&self.db)
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

pub struct EmoteSetByUserIdLoader {
	db: mongodb::Database,
}

impl EmoteSetByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("EmoteSetByUserIdLoader", Self { db })
	}
}

impl Loader for EmoteSetByUserIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<EmoteSet>;

	#[tracing::instrument(name = "EmoteSetByUserIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Vec<EmoteSet> = EmoteSet::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"owner_id": {
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

		Ok(results.into_iter().into_group_map_by(|r| r.owner_id.unwrap()))
	}
}
