use std::collections::HashMap;
use std::future::{Future, IntoFuture};
use std::sync::Arc;

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

use crate::global::Global;

pub async fn load_emotes_generic<F, Fut>(global: &Arc<Global>, loader: F) -> Result<HashMap<EmoteId, Emote>, ()>
where
	F: FnOnce() -> Fut,
	Fut: Future<Output = Result<HashMap<EmoteId, Emote>, ()>>,
{
	let mut results = loader().await?;

	let mut ids = Vec::new();

	results.retain(|_, e| {
		if e.deleted {
			false
		} else if let Some(merged) = &e.merged {
			ids.push(merged.target_id);
			false
		} else {
			true
		}
	});

	if ids.is_empty() {
		return Ok(results);
	}

	let mut i = 0;

	while !ids.is_empty() && i < 10 {
		let emotes = global.emote_by_id_loader.load_many(ids.iter().copied()).await?;

		results.extend(emotes.into_iter().filter(|(_, e)| {
			if e.deleted {
				false
			} else if let Some(merged) = &e.merged {
				ids.push(merged.target_id);
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

	Ok(results)
}

pub async fn load_emotes(
	global: &Arc<Global>,
	ids: impl IntoIterator<Item = EmoteId>,
) -> Result<HashMap<EmoteId, Emote>, ()> {
	load_emotes_generic(global, || global.emote_by_id_loader.load_many(ids)).await
}

pub async fn load_emote(global: &Arc<Global>, id: EmoteId) -> Result<Option<Emote>, ()> {
	Ok(load_emotes(global, [id]).await?.into_iter().next().map(|(_, e)| e))
}

pub async fn load_emotes_by_user_id(global: &Arc<Global>, owner_id: UserId) -> Result<HashMap<EmoteId, Emote>, ()> {
	load_emotes_generic(global, || async {
		Ok(global
			.emote_by_user_id_loader
			.load(owner_id)
			.await?
			.unwrap_or_default()
			.into_iter()
			.map(|e| (e.id, e))
			.collect())
	})
	.await
}

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
		DataLoader::new(Self { db, config })
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
