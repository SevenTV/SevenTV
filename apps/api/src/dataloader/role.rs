use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use mongodb::options::FindOptions;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::role::{Role, RoleId};
use shared::database::Collection;

pub struct RoleByIdLoader {
	db: mongodb::Database,
}

impl RoleByIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("RoleByIdLoader", Self { db })
	}
}

impl Loader for RoleByIdLoader {
	type Error = ();
	type Key = RoleId;
	type Value = Role;

	#[tracing::instrument(name = "RoleByIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Self::Value> = Role::collection(&self.db)
			.find(doc! {
				"_id": {
					"$in": keys,
				}
			})
			.with_options(FindOptions::builder().sort(doc! { "rank": 1 }).build())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().map(|r| (r.id, r)).collect())
	}
}

pub struct AllRolesLoader {
	db: mongodb::Database,
}

impl AllRolesLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("AllRolesLoader", Self { db })
	}
}

impl Loader for AllRolesLoader {
	type Error = ();
	type Key = ();
	type Value = Vec<Role>;

	#[tracing::instrument(name = "AllRolesLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Role> = Role::collection(&self.db)
			.find(doc! {})
			.with_options(FindOptions::builder().sort(doc! { "rank": 1 }).build())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok([((), results)].into_iter().collect())
	}
}
