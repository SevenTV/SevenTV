use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telementry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::{Collection, UserEditor, UserId};

pub struct UserEditorByUserIdLoader {
	pub db: mongodb::Database,
}

impl UserEditorByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserEditorByUserIdLoader", Self { db })
	}
}

impl Loader for UserEditorByUserIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<UserEditor>;

	#[tracing::instrument(name = "UserEditorByUserIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let results: Self::Value = UserEditor::collection(&self.db)
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

pub struct UserEditorByEditorIdLoader {
	pub db: mongodb::Database,
}

impl UserEditorByEditorIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("UserEditorByEditorIdLoader", Self { db })
	}
}

impl Loader for UserEditorByEditorIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<UserEditor>;

	#[tracing::instrument(name = "UserEditorByEditorIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Self::Value = UserEditor::collection(&self.db)
			.find(
				mongodb::bson::doc! {
					"editor_id": {
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

		Ok(results.into_iter().into_group_map_by(|r| r.editor_id))
	}
}
