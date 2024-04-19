use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{Collection, UserConnection, UserId};

pub struct UserConnectionByUserIdLoader {
	pub db: mongodb::Database,
}

impl UserConnectionByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserConnectionByUserIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<UserConnection>;

	#[tracing::instrument(level = "info", skip(self), fields(keys = ?keys))]
	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
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
