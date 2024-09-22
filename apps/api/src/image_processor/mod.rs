use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream::{consumer, stream};
use futures::StreamExt;
use prost::Message;
use scuffle_foundations::context::{self, ContextFutExt};
use scuffle_image_processor_proto::{event_callback, EventCallback};
use shared::database::image_set::{Image, ImageSet, ImageSetInput};
use shared::image_processor::Subject;

use crate::global::Global;
use crate::transactions::with_transaction;

mod badge;
mod emote;
mod paint_layer;
mod profile_picture;

const JETSTREAM_NAME: &str = "image-processor-callback";
const JETSTREAM_CONSUMER_NAME: &str = "image-processor-callback-consumer";

pub async fn run(global: Arc<Global>) -> Result<(), anyhow::Error> {
	let config = &global.config.api.image_processor;

	let subject = Subject::wildcard(&config.event_queue_topic_prefix);

	let stream = global
		.jetstream
		.get_or_create_stream(stream::Config {
			name: JETSTREAM_NAME.to_string(),
			max_consumers: 1,
			subjects: vec![subject],
			retention: stream::RetentionPolicy::WorkQueue,
			..Default::default()
		})
		.await
		.context("failed to create image processor callback stream")?;

	let consumer = stream
		.get_or_create_consumer(
			JETSTREAM_CONSUMER_NAME,
			consumer::pull::Config {
				name: Some(JETSTREAM_CONSUMER_NAME.to_string()),
				ack_policy: consumer::AckPolicy::Explicit,
				ack_wait: Duration::from_secs(30),
				..Default::default()
			},
		)
		.await
		.context("failed to create image processor callback consumer")?;

	let mut consumer = consumer
		.messages()
		.await
		.context("failed to get image processor callback consumer messages")?;

	while let Some(message) = consumer.next().with_context(context::Context::global()).await {
		let message = message.context("consumer closed")?.context("failed to get message")?;

		// decode
		let subject =
			match Subject::from_string(&message.subject, &global.config.api.image_processor.event_queue_topic_prefix) {
				Ok(subject) => subject,
				Err(err) => {
					tracing::warn!(error = %err, subject = %message.subject, "failed to decode subject");
					message
						.ack()
						.await
						.map_err(|e| anyhow::anyhow!(e))
						.context("failed to ack message")?;
					continue;
				}
			};

		let event_callback = match EventCallback::decode(message.payload.as_ref()) {
			Ok(callback) => callback,
			err => {
				if let Err(err) = err {
					tracing::warn!(error = %err, "failed to decode event callback");
				} else {
					tracing::warn!("malformed event callback");
				}
				tracing::error!("failed to decode message");
				message
					.ack()
					.await
					.map_err(|e| anyhow::anyhow!(e))
					.context("failed to ack message")?;
				continue;
			}
		};

		tracing::debug!(event_callback = ?event_callback, subject = ?subject, "received image processor callback event");

		// handle event
		match event_callback.event.context("missing event")? {
			event_callback::Event::Success(event) => {
				if let Err(err) = handle_success(&global, subject, event, event_callback.metadata).await {
					tracing::error!(error = %err, "failed to handle success event");
				}
			}
			event_callback::Event::Fail(event) => {
				if let Err(err) = handle_fail(&global, subject, event, event_callback.metadata).await {
					tracing::error!(error = %err, "failed to handle fail event");
				}
			}
			event_callback::Event::Cancel(event) => {
				if let Err(err) = handle_cancel(&global, subject, event, event_callback.metadata).await {
					tracing::error!(error = %err, "failed to handle cancel event");
				}
			}
			event_callback::Event::Start(event) => {
				if let Err(err) = handle_start(&global, subject, event, event_callback.metadata).await {
					tracing::error!(error = %err, "failed to handle start event");
				}
			}
		}

		// ack
		message
			.ack()
			.await
			.map_err(|e| anyhow::anyhow!(e))
			.context("failed to ack message")?;
	}

	Ok(())
}

fn event_to_image_set(event: event_callback::Success) -> anyhow::Result<ImageSet> {
	let input = event.input_metadata.context("missing input metadata")?;

	Ok(ImageSet {
		input: ImageSetInput::Image(Image {
			frame_count: input.frame_count as i32,
			width: input.width as i32,
			height: input.height as i32,
			path: input.path.map(|p| p.path).unwrap_or_default(),
			mime: input.content_type,
			size: input.size as i64,
			scale: 1,
		}),
		outputs: event
			.files
			.into_iter()
			.map(|file| Image {
				path: file.path.unwrap_or_default().path,
				mime: file.content_type,
				size: file.size as i64,
				width: file.width as i32,
				height: file.height as i32,
				frame_count: file.frame_count as i32,
				scale: file.scale.unwrap_or(1) as i32,
			})
			.collect(),
	})
}

async fn handle_success(
	global: &Arc<Global>,
	subject: Subject,
	event: event_callback::Success,
	metadata: HashMap<String, String>,
) -> anyhow::Result<()> {
	with_transaction(global, |tx| async move {
		match subject {
			Subject::Emote(id) => emote::handle_success(tx, global, id, event, metadata).await,
			Subject::ProfilePicture(id) => profile_picture::handle_success(tx, global, id, event).await,
			Subject::PaintLayer(id, layer_id) => paint_layer::handle_success(tx, global, id, layer_id, event).await,
			Subject::Badge(id) => badge::handle_success(tx, global, id, event).await,
		}
	})
	.await
	.context("transaction")
}

async fn handle_fail(
	global: &Arc<Global>,
	subject: Subject,
	event: event_callback::Fail,
	_metadata: HashMap<String, String>,
) -> anyhow::Result<()> {
	with_transaction(global, |tx| async move {
		match subject {
			Subject::Emote(id) => emote::handle_fail(tx, global, id, event).await,
			Subject::ProfilePicture(id) => profile_picture::handle_fail(tx, global, id, event).await,
			Subject::PaintLayer(id, ..) => paint_layer::handle_fail(tx, global, id, event).await,
			Subject::Badge(id) => badge::handle_fail(tx, global, id, event).await,
		}
	})
	.await
	.context("transaction")
}

async fn handle_start(
	global: &Arc<Global>,
	subject: Subject,
	_event: event_callback::Start,
	_metadata: HashMap<String, String>,
) -> anyhow::Result<()> {
	with_transaction(global, |tx| async move {
		match subject {
			Subject::Emote(id) => emote::handle_start(tx, global, id).await,
			Subject::ProfilePicture(id) => profile_picture::handle_start(tx, global, id).await,
			Subject::PaintLayer(id, ..) => paint_layer::handle_start(tx, global, id).await,
			Subject::Badge(id) => badge::handle_start(tx, global, id).await,
		}
	})
	.await
	.context("transaction")
}

async fn handle_cancel(
	global: &Arc<Global>,
	subject: Subject,
	_event: event_callback::Cancel,
	_metadata: HashMap<String, String>,
) -> anyhow::Result<()> {
	with_transaction(global, |tx| async move {
		match subject {
			Subject::Emote(id) => emote::handle_cancel(tx, global, id).await,
			Subject::ProfilePicture(id) => profile_picture::handle_cancel(tx, global, id).await,
			Subject::PaintLayer(id, ..) => paint_layer::handle_cancel(tx, global, id).await,
			Subject::Badge(id) => badge::handle_cancel(tx, global, id).await,
		}
	})
	.await
	.context("transaction")
}
