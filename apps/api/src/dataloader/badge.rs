use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::badge::{Badge, BadgeId};
use shared::database::Collection;

pub struct BadgeByIdLoader {
	db: mongodb::Database,
}

impl BadgeByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("BadgeByIdLoader", Self { db })
	}
}

impl Loader for BadgeByIdLoader {
	type Error = ();
	type Key = BadgeId;
	type Value = Badge;

	#[tracing::instrument(name = "BadgeByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Self::Value> = Self::Value::collection(&self.db)
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
