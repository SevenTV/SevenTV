use std::sync::Arc;

use itertools::Itertools;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use ulid::Ulid;
use shared::database::{EmoteSet, EmoteSetEmote};

pub struct EmoteSetByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl EmoteSetByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for EmoteSetByIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = EmoteSet;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<Self::Value> = scuffle_utils::database::query("SELECT * FROM emote_sets WHERE id = ANY($1)")
			.bind(keys)
			.build_query_as()
			.fetch_all(&self.db)
			.await
			.map_err(|e| {
				tracing::error!(err = %e, "failed to fetch badges by id");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}

pub struct EmoteSetEmoteByIdLoader {
	db: Arc<scuffle_utils::database::Pool>,
}

impl EmoteSetEmoteByIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for EmoteSetEmoteByIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<EmoteSetEmote>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<EmoteSetEmote> =
			scuffle_utils::database::query("SELECT * FROM emote_set_emotes WHERE emote_set_id = ANY($1)")
				.bind(keys)
				.build_query_as()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch badges by id");
				})?;

		Ok(results
			.into_iter()
			.group_by(|r| r.emote_set_id)
			.into_iter()
			.map(|s| (s.0, s.1.collect()))
			.collect())
	}
}
