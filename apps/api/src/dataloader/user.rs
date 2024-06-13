use futures::{TryFutureExt, TryStreamExt};
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::{Collection, User, UserId};

pub struct UserLoader {
	db: mongodb::Database,
}

impl UserLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserLoader", Self { db })
	}
}

impl Loader for UserLoader {
	type Error = ();
	type Key = UserId;
	type Value = User;

	#[tracing::instrument(name = "UserLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Vec<Self::Value> = Self::Value::collection(&self.db)
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
