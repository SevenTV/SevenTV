use std::{sync::Arc, time::Duration};

use anyhow::Context;
use async_nats::jetstream::{consumer, stream};
use futures::StreamExt;
use mongodb::bson::{doc, to_bson};
use prost::Message;
use scuffle_foundations::context::{self, ContextFutExt};
use scuffle_image_processor_proto::{event_callback, EventCallback};
use shared::database::{Collection, Emote, Image};

use crate::{global::Global, image_processor::Subject};

const JETSTREAM_NAME: &str = "image-processor-callback";
const JETSTREAM_CONSUMER_NAME: &str = "image-processor-callback-consumer";

pub async fn run(global: Arc<Global>) -> Result<(), anyhow::Error> {
	let config = &global.config().extra.api.image_processor;

	let subject = Subject::Wildcard.to_string(&config.event_queue_topic_prefix);

	let stream = global
		.jetstream()
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
		let subject = match Subject::from_string(
			&message.subject,
			&global.config().extra.api.image_processor.event_queue_topic_prefix,
		) {
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

		let event = match EventCallback::decode(message.payload.as_ref()) {
			Ok(EventCallback { event: Some(event), .. }) => event,
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

		tracing::debug!(event = ?event, subject = ?subject, "received image processor callback event");

		// handle event
		match event {
			event_callback::Event::Success(event) => {
				if let Err(err) = handle_success(&global, subject, event).await {
					tracing::error!(error = %err, "failed to handle success event");
				}
			}
			event_callback::Event::Fail(event) => {
				if let Err(err) = handle_fail(&global, subject, event).await {
					tracing::error!(error = %err, "failed to handle fail event");
				}
			}
			event_callback::Event::Cancel(_) => {
				if let Err(err) = handle_cancel(&global, subject).await {
					tracing::error!(error = %err, "failed to handle cancel event");
				}
			}
			_ => {}
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

async fn handle_success(global: &Arc<Global>, subject: Subject, event: event_callback::Success) -> anyhow::Result<()> {
	match subject {
		Subject::Emote(id) => {
			let animated = event.files.iter().any(|i| i.frame_count > 1);

			let input = event.input_metadata.context("missing input metadata")?;

			let outputs: Vec<_> = event
				.files
				.into_iter()
				.map(|i| Image {
					path: i.path,
					mime: i.content_type,
					size: i.size as u64,
					width: i.width,
					height: i.height,
					frame_count: i.frame_count,
				})
				.collect();

			Emote::collection(global.db())
				.update_one(
					doc! {
						"_id": id,
					},
					doc! {
						"$set": {
							"animated": animated,
							"image_set": {
								"input": {
									"width": input.width,
									"height": input.height,
									"frame_count": input.frame_count,
								},
								"outputs": to_bson(&outputs)?,
							},
						},
					},
					None,
				)
				.await?;
		}
		Subject::Wildcard => anyhow::bail!("received event for wildcard subject"),
	}

	Ok(())
}

async fn handle_fail(global: &Arc<Global>, subject: Subject, _event: event_callback::Fail) -> anyhow::Result<()> {
	match subject {
		Subject::Emote(id) => {
			Emote::collection(global.db())
				.delete_one(
					doc! {
						"_id": id,
					},
					None,
				)
				.await?;

			// Notify user of failure with reason

			Ok(())
		}
		Subject::Wildcard => anyhow::bail!("received event for wildcard subject"),
	}
}

async fn handle_cancel(global: &Arc<Global>, subject: Subject) -> anyhow::Result<()> {
	match subject {
		Subject::Emote(id) => {
			Emote::collection(global.db())
				.delete_one(
					doc! {
						"_id": id,
					},
					None,
				)
				.await?;

			// Notify user of cancellation

			Ok(())
		}
		Subject::Wildcard => anyhow::bail!("received event for wildcard subject"),
	}
}
