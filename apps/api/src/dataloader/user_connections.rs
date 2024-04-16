use std::collections::HashMap;
use std::sync::Arc;

use scuffle_utils::dataloader::{DataLoader, Loader, LoaderOutput};
use ulid::Ulid;
use shared::database::UserConnection;

pub struct UserConnectionsByUserIdLoader {
	pub db: Arc<scuffle_utils::database::Pool>,
}

impl UserConnectionsByUserIdLoader {
	pub fn new(db: Arc<scuffle_utils::database::Pool>) -> DataLoader<Self> {
		DataLoader::new(Self { db })
	}
}

impl Loader for UserConnectionsByUserIdLoader {
	type Error = ();
	type Key = Ulid;
	type Value = Vec<UserConnection>;

	async fn load(&self, keys: &[Self::Key]) -> LoaderOutput<Self> {
		let results: Vec<UserConnection> =
			scuffle_utils::database::query("SELECT * FROM user_connections WHERE user_id = ANY($1)")
				.bind(keys)
				.build_query_as()
				.fetch_all(&self.db)
				.await
				.map_err(|e| {
					tracing::error!(err = %e, "failed to fetch user connections by user id");
				})?;

		Ok(results.into_iter().fold(HashMap::new(), |mut map, connection| {
			map.entry(connection.user_id).or_default().push(connection);
			map
		}))
	}
}
