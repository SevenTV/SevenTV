use std::sync::Arc; 
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream::{stream, AckKind};
use bson::Document;
use futures::TryStreamExt;
use handlers::SupportedMongoCollection;
use mongodb::change_stream::event::ChangeStreamEvent;
use scuffle_foundations::context::ContextFutExt;
use scuffle_foundations::telemetry::metrics::metrics;
use scuffle_foundations::telemetry::opentelemetry::OpenTelemetrySpanExt;
use shared::database::MongoCollection;
use typesense::{EventStatus, OperationType};

use crate::global::Global;

mod handlers;

#[metrics]
mod typesense {
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::counter::Counter;
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::gauge::Gauge;
	use serde::Serialize;
	use shared::database::MongoCollection;

	pub struct Processing(&'static str);

	impl Processing {
		pub fn new<T: MongoCollection>() -> Self {
			processing(T::COLLECTION_NAME).inc();
			Self(T::COLLECTION_NAME)
		}
	}

	impl Drop for Processing {
		fn drop(&mut self) {
			processing(self.0).dec();
		}
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
	pub enum EventStatus {
		Success,
		Skipped,
		Error,
	}

	/// The operation type represented in a given change notification.
	#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
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
				Self::Other(value) => value.as_str(),
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

	pub fn event(db: &str, coll: &str, op: OperationType, status: EventStatus) -> Counter;
	pub fn processing(coll: &'static str) -> Gauge;
}

macro_rules! spawn_handler {
	(($global:ident, $stream:ident) => { $($ty:ty),* $(,)? }) => {
		{
			let mut handlers = Vec::new();
			$(
				let global = $global.clone();
				let stream = $stream.clone();
				handlers.push(tokio::spawn(async move {
					setup::<$ty>(global, stream).await.context(<$ty>::COLLECTION_NAME)
				}));
			)*
			handlers
		}
	};
}

pub async fn start(global: Arc<Global>) -> anyhow::Result<()> {
	shared::typesense::types::init_typesense(global.typesense())
		.await
		.context("failed to initialize typesense")?;

	let stream = tokio::time::timeout(
		Duration::from_secs(5),
		global.jetstream().get_or_create_stream(stream::Config {
			name: "mongo-typesense".to_string(),
			subjects: vec![
				format!("{}.>", global.config().triggers.topic),
			],
			retention: stream::RetentionPolicy::Interest,
			duplicate_window: Duration::from_secs(60),
			max_age: Duration::from_secs(60 * 60 * 24), // messages older than 24 hours are dropped
			..Default::default()
		}),
	)
	.await
	.context("create stream timeout")?
	.context("create stream")?;


	let handlers = spawn_handler!((global, stream) => {
		crate::types::mongo::DiscountCode,
		crate::types::mongo::GiftCode,
		crate::types::mongo::RedeemCode,
		crate::types::mongo::SpecialEvent,
		crate::types::mongo::Invoice,
		crate::types::mongo::Product,
		crate::types::mongo::Promotion,
		crate::types::mongo::SubscriptionTimeline,
		crate::types::mongo::SubscriptionTimelinePeriod,
		crate::types::mongo::SubscriptionCredit,
		crate::types::mongo::SubscriptionPeriod,
		crate::types::mongo::UserBanTemplate,
		crate::types::mongo::UserBan,
		crate::types::mongo::UserEditor,
		crate::types::mongo::UserRelation,
		crate::types::mongo::User,
		crate::types::mongo::AuditLog,
		crate::types::mongo::AutomodRule,
		crate::types::mongo::Badge,
		crate::types::mongo::EmoteModerationRequest,
		crate::types::mongo::EmoteSet,
		crate::types::mongo::Emote,
		crate::types::mongo::EntitlementEdge,
		crate::types::mongo::EntitlementGroup,
		crate::types::mongo::Page,
		crate::types::mongo::Paint,
		crate::types::mongo::Role,
		crate::types::mongo::Ticket,
		crate::types::mongo::TicketMessage,
	});

	let (r, _, remaining) = futures::future::select_all(handlers).await;

	for h in remaining {
		h.abort();
	}

	r??;

	tracing::info!("typesense handler exited");

	Ok(())
}

async fn setup<M: SupportedMongoCollection>(
	global: Arc<Global>,
	stream: async_nats::jetstream::stream::Stream,
) -> anyhow::Result<()> {
	let name = format!("{}::{}", global.config().triggers.seventv_database, M::COLLECTION_NAME);

	let consumer = tokio::time::timeout(
		Duration::from_secs(5),
		stream.get_or_create_consumer(
			&name.clone(),
			async_nats::jetstream::consumer::pull::Config {
				name: Some(name.clone()),
				durable_name: Some(name.clone()),
				max_ack_pending: 1_000_000,
				ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
				ack_wait: Duration::from_secs(30),
				inactive_threshold: Duration::from_secs(60 * 60 * 24),
				max_deliver: 5,
				filter_subject: format!("{}.{}.{}", global.config().triggers.topic, global.config().triggers.seventv_database, M::COLLECTION_NAME),
				..Default::default()
			},
		),
	)
	.await
	.context("create consumer timeout")?
	.context("create consumer")?;


	let ctx = scuffle_foundations::context::Context::global();

	let mut messages = tokio::time::timeout(
		Duration::from_secs(5),
		consumer.messages()
	)
	.await
	.context("get messages timeout")?
	.context("get messages")?;

	while let Some(Some(ticket)) = global.aquire_ticket().with_context(&ctx).await {
		let Some(Some(message)) = messages
			.try_next()
			.with_context(&ctx)
			.await
			.transpose()
			.context("get message")?
		else {
			break;
		};

		let metrics = typesense::Processing::new::<M>();
		let global = global.clone();

		tokio::spawn(
			async move {
				handle_message::<M>(&global, message).await;
				global.incr_request_count();
				drop((ticket, metrics));
			}
			.with_context(scuffle_foundations::context::Context::global()),
		);
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

	let result = handlers::process::<M>(&global, message).await;

	let status = match &result {
		Ok(true) => EventStatus::Success,
		Ok(false) => EventStatus::Skipped,
		Err(_) => EventStatus::Error,
	};

	tracing::debug!(status = ?status, "handled typesense event");
	typesense::event(&db, &coll, operation, status).inc();

	result.map(|_| ())
}

#[tracing::instrument(skip_all, fields(collection = M::COLLECTION_NAME))]
async fn handle_message<M: SupportedMongoCollection>(global: &Arc<Global>, message: async_nats::jetstream::Message) {
	tracing::Span::current().make_root();

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

	let mut handle_fut = handle::<M>(&global, event);
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
