use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::bson::oid::ObjectId;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{Collection, EmoteSet, EmoteSetEmote};

pub struct EmoteSetByIdLoader {
	db: mongodb::Database,
}

impl EmoteSetByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for EmoteSetByIdLoader {
	type Error = ();
	type Key = ObjectId;
	type Value = EmoteSet;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
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

pub struct EmoteSetEmoteByIdLoader {
	db: mongodb::Database,
}

impl EmoteSetEmoteByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for EmoteSetEmoteByIdLoader {
	type Error = ();
	type Key = ObjectId;
	type Value = Vec<EmoteSetEmote>;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<EmoteSetEmote> = EmoteSetEmote::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"emote_set_id": {
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

		Ok(results.into_iter().into_group_map_by(|r| r.emote_set_id))
	}
}

pub struct EmoteSetByUserIdLoader {
	db: mongodb::Database,
}

impl EmoteSetByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for EmoteSetByUserIdLoader {
	type Error = ();
	type Key = ObjectId;
	type Value = Vec<EmoteSet>;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
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
