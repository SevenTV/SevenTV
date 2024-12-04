use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream::{stream, AckKind};
use bson::Document;
use futures::TryStreamExt;
use handlers::SupportedMongoCollection;
use mongodb::change_stream::event::ChangeStreamEvent;
use scuffle_context::ContextFutExt;
use shared::database::MongoCollection;
use typesense::{EventStatus, OperationType};

use crate::global::Global;

mod handlers;

#[scuffle_metrics::metrics]
mod typesense {
	use scuffle_metrics::{CounterU64, MetricEnum, UpDownCounterI64};
	use shared::database::MongoCollection;

	pub struct Processing(&'static str);

	impl Processing {
		pub fn new<T: MongoCollection>() -> Self {
			processing(T::COLLECTION_NAME).incr();
			Self(T::COLLECTION_NAME)
		}
	}

	impl Drop for Processing {
		fn drop(&mut self) {
			processing(self.0).decr();
		}
	}

	#[derive(Debug, Clone, Copy, MetricEnum)]
	pub enum EventStatus {
		Success,
		Skipped,
		Error,
	}

	/// The operation type represented in a given change notification.
	#[derive(Debug, Clone)]
	pub enum OperationType {
		Insert,
		Update,
		Replace,
		Delete,
		Drop,
		Rename,
		DropDatabase,
		Invalidate,
		Other(String),
	}

	impl OperationType {
		pub fn as_str(&self) -> &str {
			match self {
				Self::Insert => "Insert",
				Self::Update => "Update",
				Self::Replace => "Replace",
				Self::Delete => "Delete",
				Self::Drop => "Drop",
				Self::Rename => "Rename",
				Self::DropDatabase => "DropDatabase",
				Self::Invalidate => "Invalidate",
				Self::Other(value) => value,
			}
		}
	}

	impl From<OperationType> for scuffle_metrics::opentelemetry::Value {
		fn from(value: OperationType) -> Self {
			match value {
				OperationType::Insert => scuffle_metrics::opentelemetry::Value::from("Insert"),
				OperationType::Update => scuffle_metrics::opentelemetry::Value::from("Update"),
				OperationType::Replace => scuffle_metrics::opentelemetry::Value::from("Replace"),
				OperationType::Delete => scuffle_metrics::opentelemetry::Value::from("Delete"),
				OperationType::Drop => scuffle_metrics::opentelemetry::Value::from("Drop"),
				OperationType::Rename => scuffle_metrics::opentelemetry::Value::from("Rename"),
				OperationType::DropDatabase => scuffle_metrics::opentelemetry::Value::from("DropDatabase"),
				OperationType::Invalidate => scuffle_metrics::opentelemetry::Value::from("Invalidate"),
				OperationType::Other(value) => scuffle_metrics::opentelemetry::Value::from(value.clone()),
			}
		}
	}

	impl From<mongodb::change_stream::event::OperationType> for OperationType {
		fn from(value: mongodb::change_stream::event::OperationType) -> Self {
			match value {
				mongodb::change_stream::event::OperationType::Insert => Self::Insert,
				mongodb::change_stream::event::OperationType::Update => Self::Update,
				mongodb::change_stream::event::OperationType::Replace => Self::Replace,
				mongodb::change_stream::event::OperationType::Delete => Self::Delete,
				mongodb::change_stream::event::OperationType::Drop => Self::Drop,
				mongodb::change_stream::event::OperationType::Rename => Self::Rename,
				mongodb::change_stream::event::OperationType::DropDatabase => Self::DropDatabase,
				mongodb::change_stream::event::OperationType::Invalidate => Self::Invalidate,
				mongodb::change_stream::event::OperationType::Other(value) => Self::Other(value),
				r => Self::Other(format!("{:?}", r)),
			}
		}
	}

	pub fn event(db: &str, coll: &str, op: OperationType, status: EventStatus) -> CounterU64;
	fn processing(coll: &'static str) -> UpDownCounterI64;
}

pub async fn run(global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
	shared::typesense::types::init_typesense(&global.typesense)
		.await
		.context("failed to initialize typesense")?;

	let subject = shared::nats::ChangeStreamSubject::new(&global.config.triggers.nats_prefix);

	let config = stream::Config {
		name: subject.name(),
		subjects: vec![subject.name()],
		retention: stream::RetentionPolicy::WorkQueue,
		duplicate_window: Duration::from_secs(15),
		max_age: Duration::from_secs(60 * 60 * 24), // messages older than 24 hours are dropped
		max_bytes: 1024 * 1024 * 1024 * 100,        // 100GB max
		storage: stream::StorageType::File,
		..Default::default()
	};

	let stream = tokio::time::timeout(Duration::from_secs(5), global.jetstream.get_or_create_stream(config.clone()))
		.await
		.context("create stream timeout")?
		.context("create stream")?;

	tokio::time::timeout(Duration::from_secs(5), global.jetstream.update_stream(config.clone()))
		.await
		.context("update stream timeout")?
		.context("update stream")?;

	setup(&global, stream, &ctx).await?;

	tracing::info!("typesense handler exited");

	Ok(())
}

async fn setup(
	global: &Arc<Global>,
	stream: async_nats::jetstream::stream::Stream,
	ctx: &scuffle_context::Context,
) -> anyhow::Result<()> {
	let config = async_nats::jetstream::consumer::pull::Config {
		name: Some("change-stream".to_string()),
		durable_name: None,
		max_ack_pending: 1_000_000,
		ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
		ack_wait: Duration::from_secs(30),
		inactive_threshold: Duration::from_secs(60 * 60 * 24),
		max_deliver: 5,
		..Default::default()
	};

	let consumer = tokio::time::timeout(
		Duration::from_secs(5),
		stream.get_or_create_consumer("change-stream", config.clone()),
	)
	.await
	.context("create consumer timeout")?
	.context("create consumer")?;

	tokio::time::timeout(Duration::from_secs(5), stream.update_consumer(config.clone()))
		.await
		.context("update consumer timeout")?
		.context("update consumer")?;

	let mut messages = tokio::time::timeout(Duration::from_secs(5), consumer.messages())
		.await
		.context("get messages timeout")?
		.context("get messages")?;

	while let Some(Some(ticket)) = global.aquire_ticket().with_context(ctx).await {
		let Some(Some(message)) = messages
			.try_next()
			.with_context(ctx)
			.await
			.transpose()
			.context("get message")?
		else {
			break;
		};

		let Some(collection) = message
			.headers
			.as_ref()
			.and_then(|headers| headers.get("collection").map(|s| s.as_str()))
		else {
			message.ack_with(AckKind::Term).await.ok();
			continue;
		};

		macro_rules! match_collection {
			($str:ident, $ctx:ident => { $($collection:ty),*$(,)? }) => {
				match $str {
					$(
						<$collection>::COLLECTION_NAME => {
							let metrics = typesense::Processing::new::<$collection>();
							let global = global.clone();
							let ctx = ctx.clone();

							tokio::spawn(
								async move {
									handle_message::<$collection>(&global, message, &ctx).await;
									global.incr_request_count();
									drop((ticket, metrics));
								},
							);
						}
					),*
					crate::types::mongo::UserProfilePicture::COLLECTION_NAME
						| crate::types::mongo::UserSession::COLLECTION_NAME
						| crate::types::mongo::WebhookEvent::COLLECTION_NAME
						| crate::types::mongo::CronJob::COLLECTION_NAME => {
						message.ack_with(AckKind::Term).await.ok();
					}
					_ => {
						tracing::warn!(collection = %$str, "unknown collection");
						message.ack_with(AckKind::Term).await.ok();
					}
				}
			};
		}

		match_collection! {
			collection, ctx => {
				crate::types::mongo::RedeemCode,
				crate::types::mongo::SpecialEvent,
				crate::types::mongo::Invoice,
				crate::types::mongo::Product,
				crate::types::mongo::SubscriptionProduct,
				crate::types::mongo::SubscriptionPeriod,
				crate::types::mongo::UserBan,
				crate::types::mongo::UserEditor,
				crate::types::mongo::User,
				crate::types::mongo::StoredEvent,
				crate::types::mongo::Badge,
				crate::types::mongo::EmoteModerationRequest,
				crate::types::mongo::EmoteSet,
				crate::types::mongo::Emote,
				crate::types::mongo::EntitlementEdge,
				crate::types::mongo::Paint,
				crate::types::mongo::Role,
				crate::types::mongo::Ticket,
				crate::types::mongo::TicketMessage,
				crate::types::mongo::Subscription,
			}
		}
	}

	Ok(())
}

#[tracing::instrument(skip_all, fields(db, coll, operation))]
async fn handle<M: SupportedMongoCollection>(
	global: &Arc<Global>,
	message: ChangeStreamEvent<Document>,
) -> anyhow::Result<()> {
	let db = message.to.as_ref().map(|c| c.db.as_str()).unwrap_or("").to_string();
	let coll = message.to.as_ref().and_then(|c| c.coll.as_deref()).unwrap_or("").to_string();
	let operation = OperationType::from(message.operation_type.clone());

	tracing::Span::current().record("db", &db);
	tracing::Span::current().record("coll", &coll);
	tracing::Span::current().record("operation", operation.as_str());

	let result = handlers::process::<M>(global, message).await;

	let status = match &result {
		Ok(true) => EventStatus::Success,
		Ok(false) => EventStatus::Skipped,
		Err(_) => EventStatus::Error,
	};

	tracing::debug!(status = ?status, "handled typesense event");
	typesense::event(&db, &coll, operation, status).incr();

	result.map(|_| ())
}

#[tracing::instrument(skip_all, fields(collection = M::COLLECTION_NAME))]
async fn handle_message<M: SupportedMongoCollection>(
	global: &Arc<Global>,
	message: async_nats::jetstream::Message,
	ctx: &scuffle_context::Context,
) {
	let event: ChangeStreamEvent<Document> = match serde_json::from_slice(&message.payload) {
		Ok(event) => event,
		Err(err) => {
			global.report_error();

			tracing::error!("failed to parse message: {:#}", err);
			if let Err(err) = message.ack_with(AckKind::Nak(Some(Duration::from_secs(5)))).await {
				tracing::error!("failed to ack message: {:#}", err);
			}

			return;
		}
	};

	let mut handle_fut = handle::<M>(global, event).with_context(ctx);
	let mut handle_fut = std::pin::pin!(handle_fut);

	let r = loop {
		tokio::select! {
			r = &mut handle_fut => {
				break r;
			},
			_ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
				if let Err(err) = message.ack_with(AckKind::Progress).await {
					tracing::error!("failed to ack message: {:#}", err);
					global.report_error();
					return;
				}
			},
		}
	};

	let Some(r) = r else {
		return;
	};

	if let Err(err) = r {
		global.report_error();

		tracing::error!("failed to handle event: {:#}", err);
		if let Err(err) = message.ack_with(AckKind::Nak(Some(Duration::from_secs(5)))).await {
			tracing::error!("failed to ack message: {:#}", err);
		}

		return;
	} else if let Err(err) = message.ack().await {
		global.report_error();
		tracing::error!("failed to ack message: {:#}", err);
		return;
	}
}
