use std::future::IntoFuture;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::header::NATS_MESSAGE_ID;
use async_nats::jetstream::context::PublishAckFuture;
use async_nats::{HeaderMap, HeaderValue};
use bytes::Bytes;
use mongodb::change_stream::event::OperationType;
use scuffle_foundations::context::ContextFutExt;
use scuffle_foundations::telemetry::metrics::metrics;
use tokio_stream::StreamExt;

use crate::global::Global;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
enum Operation {
	Insert,
	Update,
	Replace,
	Delete,
	Drop,
	Rename,
	DropDatabase,
	Invalidate,
	SearchUpdate,
	Other(String),
}

impl From<OperationType> for Operation {
	fn from(value: OperationType) -> Self {
		match value {
			OperationType::Insert => Operation::Insert,
			OperationType::Update => Operation::Update,
			OperationType::Replace => Operation::Replace,
			OperationType::Delete => Operation::Delete,
			OperationType::Invalidate => Operation::Invalidate,
			OperationType::Drop => Operation::Drop,
			OperationType::DropDatabase => Operation::DropDatabase,
			OperationType::Rename => Operation::Rename,
			OperationType::Other(value) => Operation::Other(value),
			r => Operation::Other(format!("{:?}", r)),
		}
	}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
enum PublishStatus {
	Success,
	Error,
	Timeout,
}

#[metrics]
mod mongo_change_stream {
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::counter::Counter;

	use super::{Operation, PublishStatus};

	/// The number of change stream events for a given database, collection, and
	/// operation.
	pub fn event(database: &str, collection: &str, operation: Operation) -> Counter;

	pub fn publish(status: PublishStatus) -> Counter;
}

pub async fn start(global: Arc<Global>) -> anyhow::Result<()> {
	let ctx = scuffle_foundations::context::Context::global();

	let (tx, mut rx) = tokio::sync::mpsc::channel::<PublishAckFuture>(global.config.back_pressure);

	struct DropHandle<T>(tokio::task::JoinHandle<T>);

	impl<T> Drop for DropHandle<T> {
		fn drop(&mut self) {
			self.0.abort();
		}
	}

	let _handle = DropHandle(tokio::spawn(
		async move {
			let mut errors = 0;
			while let Some(ack) = rx.recv().await {
				if let Err(err) = ack.await {
					tracing::warn!("failed to ack message: {:#}", err);
					errors += 1;
					if errors > 5 {
						tracing::error!("too many errors, stopping");
						// We need to sleep here so we dont overwhelm the sender
						tokio::time::sleep(std::time::Duration::from_secs(1)).await;
					}
				}

				if rx.is_closed() {
					break;
				}
			}
		}
		.with_context(ctx.clone()),
	));

	let Some(mut watch_stream) = tokio::time::timeout(Duration::from_secs(5), global.database.watch().into_future())
		.with_context(&ctx)
		.await
		.transpose()
		.context("watch stream timeout")?
		.transpose()
		.context("watch stream failed")?
	else {
		return Ok(());
	};

	tracing::info!("watching for changes");

	let subject = shared::nats::ChangeStreamSubject::new(&global.config.nats_prefix);

	global
		.jetstream
		.get_or_create_stream(async_nats::jetstream::stream::Config {
			name: subject.name(),
			subjects: vec![subject.wildcard()],
			retention: async_nats::jetstream::stream::RetentionPolicy::Interest,
			duplicate_window: std::time::Duration::from_secs(60),
			storage: async_nats::jetstream::stream::StorageType::File,
			..Default::default()
		})
		.await
		.context("create stream")?;

	while let Some(Some(event)) = watch_stream
		.try_next()
		.with_context(&ctx)
		.await
		.transpose()
		.context("watch stream event")?
	{
		let Some(ns) = event.ns.as_ref() else {
			continue;
		};

		let Some(collection) = ns.coll.as_deref() else {
			continue;
		};

		if event
			.update_description
			.as_ref()
			.is_some_and(|ud| ud.updated_fields.iter().all(|(f, _)| f == "search_updated_at"))
		{
			mongo_change_stream::event(&ns.db, collection, Operation::SearchUpdate).inc();
			continue;
		}

		mongo_change_stream::event(&ns.db, collection, event.operation_type.clone().into()).inc();

		let id = {
			let id = serde_json::to_value(&event.id).context("serialize event id")?;
			let mut hash = std::hash::DefaultHasher::new();
			id.hash(&mut hash);
			hash.finish()
		};

		let event = serde_json::to_vec(&event).context("serialize event")?;

		let headers = HeaderMap::from_iter(std::iter::once((NATS_MESSAGE_ID, HeaderValue::from(id))));

		if !publish_nats(&global, &tx, subject.topic(&ns.db, collection), headers, Bytes::from(event)).await {
			anyhow::bail!("failed to publish event");
		}
	}

	Ok(())
}

async fn publish_nats(
	global: &Arc<Global>,
	tx: &tokio::sync::mpsc::Sender<PublishAckFuture>,
	topic: String,
	headers: HeaderMap,
	payload: Bytes,
) -> bool {
	let ctx = scuffle_foundations::context::Context::global();

	let make = || {
		tokio::time::timeout(Duration::from_secs(5), async {
			global
				.jetstream
				.publish_with_headers(topic.clone(), headers.clone(), payload.clone())
				.await
				.context("publish event")
		})
	};

	for _ in 0..10 {
		let Some(result) = make().with_context(&ctx).await else {
			return true;
		};

		match result {
			Ok(Ok(ack)) => {
				mongo_change_stream::publish(PublishStatus::Success).inc();
				if tx.send(ack).await.is_err() {
					tracing::error!("failed to ack message");
					return false;
				}

				return true;
			}
			Ok(Err(err)) => {
				tracing::warn!("failed to publish event: {:#}", err);
				mongo_change_stream::publish(PublishStatus::Error).inc();
			}
			Err(_) => {
				tracing::warn!("failed to publish event: timedout");
				mongo_change_stream::publish(PublishStatus::Timeout).inc();
			}
		}

		tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	}

	false
}
