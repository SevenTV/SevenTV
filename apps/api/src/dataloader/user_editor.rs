use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::batcher::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::batcher::BatcherConfig;
use shared::database::queries::filter;
use shared::database::user::editor::{UserEditor, UserEditorId};
use shared::database::user::UserId;
use shared::database::MongoCollection;

pub struct UserEditorByUserIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl UserEditorByUserIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "UserEditorByUserIdLoader".to_string(),
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

impl Loader for UserEditorByUserIdLoader {
	type Key = UserId;
	type Value = Vec<UserEditor>;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Self::Value = UserEditor::collection(&self.db)
			.find(filter::filter! {
				UserEditor {
					#[query(rename = "_id", flatten)]
					id: UserEditorId {
						#[query(selector = "in")]
						user_id: keys,
					},
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|r| r.id.user_id))
	}
}

pub struct UserEditorByEditorIdLoader {
	db: mongodb::Database,
	config: BatcherConfig,
}

impl UserEditorByEditorIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		Self::new_with_config(
			db,
			BatcherConfig {
				name: "UserEditorByEditorIdLoader".to_string(),
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

impl Loader for UserEditorByEditorIdLoader {
	type Key = UserId;
	type Value = Vec<UserEditor>;

	fn config(&self) -> BatcherConfig {
		self.config.clone()
	}

	#[tracing::instrument(skip_all, fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		let results: Self::Value = UserEditor::collection(&self.db)
			.find(filter::filter! {
				UserEditor {
					#[query(rename = "_id", flatten)]
					id: UserEditorId {
						#[query(selector = "in")]
						editor_id: keys,
					},
				}
			})
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(results.into_iter().into_group_map_by(|r| r.id.editor_id))
	}
}
