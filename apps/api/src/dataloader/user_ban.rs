use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use mongodb::bson::DateTime;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::{Collection, UserBan, UserId};

pub struct ActiveUserBanByUserIdLoader {
	pub db: mongodb::Database,
}

impl ActiveUserBanByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("ActiveUserBanByUserIdLoader", Self { db })
	}
}

impl Loader for ActiveUserBanByUserIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<UserBan>;

	#[tracing::instrument(name = "ActiveUserBanByUserIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Self::Value = UserBan::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"user_id": {
						"$in": keys,
					},
					"$or": [
						{ "expires_at": Option::<DateTime>::None },
						{ "expires_at": { "$gt": chrono::Utc::now() } },
					],
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
