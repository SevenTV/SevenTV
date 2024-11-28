use std::collections::HashMap;
use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::options::ReadPreference;
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::emote::{Emote, EmoteId};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::queries::filter;
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct MergedResult {
	pub emotes: HashMap<EmoteId, Emote>,
	pub merged_ids: HashMap<EmoteId, EmoteId>,
}

impl MergedResult {
	pub fn get(&self, mut id: EmoteId) -> Option<&Emote> {
		for _ in 0..10 {
			match self.merged_ids.get(&id) {
				Some(target_id) => id = *target_id,
				None => return self.emotes.get(&id),
			}
		}

		None
	}
}

pub trait EmoteByIdLoaderExt {
	async fn load_exclude_deleted(&self, id: EmoteId) -> Result<Option<Emote>, ()>;
	async fn load_many_exclude_deleted(&self, ids: impl IntoIterator<Item = EmoteId>)
		-> Result<HashMap<EmoteId, Emote>, ()>;
	async fn load_many_merged(&self, ids: impl IntoIterator<Item = EmoteId>) -> Result<MergedResult, ()>;
}

impl EmoteByIdLoaderExt for DataLoader<EmoteByIdLoader> {
	async fn load_exclude_deleted(&self, id: EmoteId) -> Result<Option<Emote>, ()> {
		let Some(result) = self.load(id).await? else {
			return Ok(None);
		};

		if result.deleted || result.merged.is_some() {
			Ok(None)
		} else {
			Ok(Some(result))
		}
	}

	async fn load_many_exclude_deleted(
		&self,
		ids: impl IntoIterator<Item = EmoteId>,
	) -> Result<HashMap<EmoteId, Emote>, ()> {
		let results = self.load_many(ids).await?;

		Ok(results
			.into_iter()
			.filter(|(_, e)| !e.deleted && e.merged.is_none())
			.collect())
	}

	async fn load_many_merged(&self, ids: impl IntoIterator<Item = EmoteId>) -> Result<MergedResult, ()> {
		let mut emotes = self.load_many(ids).await?;

		let mut merged_ids = HashMap::new();

		emotes.retain(|_, e| {
			if e.deleted {
				false
			} else if let Some(merged) = &e.merged {
				merged_ids.insert(e.id, merged.target_id);
				false
			} else {
				true
			}
		});

		if merged_ids.is_empty() {
			return Ok(MergedResult { emotes, merged_ids });
		}

		let mut i = 0;

		let mut ids = merged_ids.values().copied().collect::<Vec<_>>();

		while !ids.is_empty() && i < 10 {
			let additional_emotes = self.load_many(ids.drain(..)).await?;

			emotes.extend(additional_emotes.into_iter().filter(|(_, e)| {
				if e.deleted {
					false
				} else if let Some(merged) = &e.merged {
					merged_ids.insert(e.id, merged.target_id);
					if !merged_ids.contains_key(&merged.target_id) {
						ids.push(merged.target_id);
					}
					false
				} else {
					true
				}
			}));

			i += 1;
		}

		if !ids.is_empty() {
			tracing::warn!(ids = ?ids, "failed to load emotes due to too many merges");
		}

		Ok(MergedResult { emotes, merged_ids })
	}
}

pub struct EmoteByUserIdLoader {
	db: mongodb::Database,
	name: String,
}

impl EmoteByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			"EmoteByUserIdLoader".to_string(),
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

impl DataLoaderFetcher for EmoteByUserIdLoader {
	type Key = UserId;
	type Value = Vec<Emote>;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let results: Vec<_> = Emote::collection(&self.db)
			.find(filter::filter! {
				Emote {
					#[query(selector = "in", serde)]
					owner_id: &keys,
					deleted: false,
					#[query(serde)]
					merged: &None,
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

		Some(results.into_iter().into_group_map_by(|e| e.owner_id))
	}
}

pub struct EmoteByIdLoader {
	db: mongodb::Database,
	name: String,
}

impl EmoteByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(db, "EmoteByIdLoader".to_string(), 500, std::time::Duration::from_millis(5))
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

impl DataLoaderFetcher for EmoteByIdLoader {
	type Key = EmoteId;
	type Value = Emote;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let results: Vec<Emote> = Emote::collection(&self.db)
			.find(filter::filter! {
				Emote {
					#[query(rename = "_id", selector = "in", serde)]
					id: &keys,
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

		Some(results.into_iter().map(|r| (r.id(), r)).collect())
	}
}
