use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream::{consumer, stream};
use futures::StreamExt;
use prost::Message;
use scuffle_foundations::context::{self, ContextFutExt};
use scuffle_image_processor_proto::EventCallback;
use shared::image_processor::Subject;

use crate::global::Global;

const JETSTREAM_NAME: &str = "image-processor-callback";
const JETSTREAM_CONSUMER_NAME: &str = "image-processor-callback-consumer";

pub async fn run(global: Arc<Global>) -> Result<(), anyhow::Error> {
	if !global.config.should_run_cosmetics() {
		tracing::info!("image processor callback job is disabled");
		return Ok(());
	}

	let config = &global.config.image_processor;

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

	// until context is shutdown
	while let Some(message) = consumer.next().with_context(context::Context::global()).await {
		let message = message.context("consumer closed")?.context("failed to get message")?;

		// decode
		let subject = match Subject::from_string(&message.subject, &global.config.image_processor.event_queue_topic_prefix)
		{
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

		let tasks = global.all_tasks.lock().await;
		let Some(task) = tasks.get(&event_callback.id) else {
			tracing::warn!(event_callback = ?event_callback, "task not found");
			continue;
		};

		task.send(event_callback).await.context("failed to send event callback")?;

		// ack
		message
			.ack()
			.await
			.map_err(|e| anyhow::anyhow!(e))
			.context("failed to ack message")?;
	}

	Ok(())
}
