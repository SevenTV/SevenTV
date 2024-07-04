use std::future::IntoFuture;

use bson::doc;
use futures::{TryFutureExt, TryStreamExt};
use itertools::Itertools;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::audit_log::{AuditLog, AuditLogData};
use shared::database::user::UserId;
use shared::database::{Collection, Id};

pub struct AuditLogByTargetIdLoader {
	db: mongodb::Database,
}

impl AuditLogByTargetIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("AuditLogByTargetIdLoader", Self { db })
	}
}

impl Loader for AuditLogByTargetIdLoader {
	type Error = ();
	type Key = Id<()>;
	type Value = Vec<AuditLog>;

	#[tracing::instrument(name = "AuditLogByTargetIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let logs: Vec<AuditLog> = AuditLog::collection(&self.db)
			.find(doc! {
				"data.target_id": {
					"$in": keys,
				},
			})
			.sort(doc! { "_id": -1 })
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(logs.into_iter().into_group_map_by(|l| match l.data {
			AuditLogData::Emote { target_id, .. } => target_id.cast(),
			AuditLogData::EmoteSet { target_id, .. } => target_id.cast(),
			AuditLogData::Ticket { target_id, .. } => target_id.cast(),
			AuditLogData::User { target_id, .. } => target_id.cast(),
		}))
	}
}

pub struct AuditLogByActorIdLoader {
	db: mongodb::Database,
}

impl AuditLogByActorIdLoader {
	pub fn new(db: mongodb::Database) -> DataLoader<Self> {
		DataLoader::new("AuditLogByActorIdLoader", Self { db })
	}
}

impl Loader for AuditLogByActorIdLoader {
	type Error = ();
	type Key = UserId;
	type Value = Vec<AuditLog>;

	#[tracing::instrument(name = "EmoteSetActivityByEmoteIdLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
		tracing::Span::current().make_root();

		let logs: Vec<AuditLog> = AuditLog::collection(&self.db)
			.find(doc! {
				"actor_id": {
					"$in": keys,
				},
			})
			.sort(doc! { "_id": -1 })
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
			})?;

		Ok(logs
			.into_iter()
			.filter(|l| l.actor_id.is_some())
			.into_group_map_by(|l| l.actor_id.unwrap()))
	}
}
