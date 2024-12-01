use std::future::IntoFuture;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::header::NATS_MESSAGE_ID;
use async_nats::jetstream::context::PublishAckFuture;
use async_nats::{HeaderMap, HeaderName, HeaderValue};
use bytes::Bytes;
use mongodb::change_stream::event::OperationType;
use scuffle_context::ContextFutExt;
use scuffle_metrics::MetricEnum;
use tokio_stream::StreamExt;

use crate::global::Global;

#[derive(Debug, Clone)]
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

impl From<Operation> for scuffle_bootstrap_telemetry::opentelemetry::Value {
	fn from(value: Operation) -> Self {
		match value {
			Operation::Other(value) => scuffle_bootstrap_telemetry::opentelemetry::Value::from(value),
			Operation::Insert => scuffle_bootstrap_telemetry::opentelemetry::Value::from("insert"),
			Operation::Update => scuffle_bootstrap_telemetry::opentelemetry::Value::from("update"),
			Operation::Replace => scuffle_bootstrap_telemetry::opentelemetry::Value::from("replace"),
			Operation::Delete => scuffle_bootstrap_telemetry::opentelemetry::Value::from("delete"),
			Operation::Drop => scuffle_bootstrap_telemetry::opentelemetry::Value::from("drop"),
			Operation::Rename => scuffle_bootstrap_telemetry::opentelemetry::Value::from("rename"),
			Operation::DropDatabase => scuffle_bootstrap_telemetry::opentelemetry::Value::from("drop_database"),
			Operation::Invalidate => scuffle_bootstrap_telemetry::opentelemetry::Value::from("invalidate"),
			Operation::SearchUpdate => scuffle_bootstrap_telemetry::opentelemetry::Value::from("search_update"),
		}
	}
}

#[derive(Debug, Clone, MetricEnum)]
enum PublishStatus {
	Success,
	Error,
	Timeout,
}

#[scuffle_metrics::metrics]
mod mongo_change_stream {
	use scuffle_metrics::CounterU64;

	use super::{Operation, PublishStatus};

	/// The number of change stream events for a given database, collection, and
	/// operation.
	pub fn event(database: &str, collection: &str, operation: Operation) -> CounterU64;

	/// The number of times a change stream event was published to NATS.
	pub fn publish(status: PublishStatus) -> CounterU64;
}

pub async fn run(global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
	let (tx, mut rx) = tokio::sync::mpsc::channel::<PublishAckFuture>(global.config.back_pressure);

	struct DropHandle<T>(tokio::task::JoinHandle<T>);

	impl<T> Drop for DropHandle<T> {
		fn drop(&mut self) {
			self.0.abort();
		}
	}

	let _handle = DropHandle(tokio::spawn(async move {
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
	}));

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
			subjects: vec![subject.name()],
			retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
			duplicate_window: std::time::Duration::from_secs(15),
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
			mongo_change_stream::event(&ns.db, collection, Operation::SearchUpdate).incr();
			continue;
		}

		mongo_change_stream::event(&ns.db, collection, event.operation_type.clone().into()).incr();

		let id = {
			let id = serde_json::to_value(&event.id).context("serialize event id")?;
			let mut hash = std::hash::DefaultHasher::new();
			id.hash(&mut hash);
			hash.finish()
		};

		let event = serde_json::to_vec(&event).context("serialize event")?;

		let headers = HeaderMap::from_iter([
			(NATS_MESSAGE_ID, HeaderValue::from(id)),
			(HeaderName::from_static("collection"), HeaderValue::from(collection)),
		]);

		if !publish_nats(&global, &tx, subject.name(), headers, Bytes::from(event), &ctx).await {
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
	ctx: &scuffle_context::Context,
) -> bool {
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
		let Some(result) = make().with_context(ctx).await else {
			return true;
		};

		match result {
			Ok(Ok(ack)) => {
				mongo_change_stream::publish(PublishStatus::Success).incr();
				if tx.send(ack).await.is_err() {
					tracing::error!("failed to ack message");
					return false;
				}

				return true;
			}
			Ok(Err(err)) => {
				tracing::warn!("failed to publish event: {:#}", err);
				mongo_change_stream::publish(PublishStatus::Error).incr();
			}
			Err(_) => {
				tracing::warn!("failed to publish event: timedout");
				mongo_change_stream::publish(PublishStatus::Timeout).incr();
			}
		}

		tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	}

	false
}
