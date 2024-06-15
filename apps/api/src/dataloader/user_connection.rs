use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::{Collection, UserConnection, UserId};

pub struct UserConnectionByUserIdLoader {
	pub db: mongodb::Database,
}

impl UserConnectionByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserConnectionByUserIdLoader", Self { db })
	}
}

impl Loader for UserConnectionByUserIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<UserConnection>;

	#[tracing::instrument(name = "UserConnectionByUserIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Self::Value = UserConnection::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"user_id": {
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

		Ok(results.into_iter().into_group_map_by(|r| r.user_id))
	}
}

pub struct UserConnectionByPlatformIdLoader {
	pub db: mongodb::Database,
}

impl UserConnectionByPlatformIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserConnectionByPlatformIdLoader", Self { db })
	}
}

impl Loader for UserConnectionByPlatformIdLoader {
	type Error = ();
	type Key = String;
	type Value = UserConnection;

	#[tracing::instrument(name = "UserConnectionByPlatformIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Self::Value> = UserConnection::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"platform_id": {
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

		Ok(results.into_iter().map(|r| (r.platform_id.clone(), r)).collect())
	}
}
