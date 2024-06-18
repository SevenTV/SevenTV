use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream::{consumer, stream};
use futures::StreamExt;
use mongodb::bson::{doc, to_bson};
use prost::Message;
use scuffle_foundations::context::{self, ContextFutExt};
use scuffle_image_processor_proto::{event_callback, EventCallback};
use shared::database::{Badge, Collection, Emote, Image, Paint, PaintLayerId, User};
use shared::image_processor::Subject;

use crate::global::Global;

const JETSTREAM_NAME: &str = "image-processor-callback";
const JETSTREAM_CONSUMER_NAME: &str = "image-processor-callback-consumer";

pub async fn run(global: Arc<Global>) -> Result<(), anyhow::Error> {
	if !global.config().should_run_cosmetics() {
		tracing::info!("image processor callback job is disabled");
		return Ok(());
	}

	let config = &global.config().extra.image_processor;

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

	let mut processed_tasks = HashSet::new();

	// until context is shutdown
	while let Some(message) = consumer.next().with_context(context::Context::global()).await {
		let message = message.context("consumer closed")?.context("failed to get message")?;

		// decode
		let subject = match Subject::from_string(
			&message.subject,
			&global.config().extra.image_processor.event_queue_topic_prefix,
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
			event_callback::Event::Start(_) => {
				// ack
				message
					.ack()
					.await
					.map_err(|e| anyhow::anyhow!(e))
					.context("failed to ack message")?;
				continue;
			}
			event_callback::Event::Success(event) => {
				if let Err(err) = handle_success(&global, subject, event_callback.metadata, event).await {
					tracing::error!(error = %err, "failed to handle success event");
				}
			}
			event_callback::Event::Fail(event) => {
				if let Err(err) = handle_fail(&global, subject, event_callback.metadata, event).await {
					tracing::error!(error = %err, "failed to handle fail event");
				}
			}
			event_callback::Event::Cancel(_) => {
				if let Err(err) = handle_cancel(&global, subject, event_callback.metadata).await {
					tracing::error!(error = %err, "failed to handle cancel event");
				}
			}
		}

		processed_tasks.insert(event_callback.id.clone());

		// ack
		message
			.ack()
			.await
			.map_err(|e| anyhow::anyhow!(e))
			.context("failed to ack message")?;

		// check if we should stop
		// total_tasks is always sorted
		let all_tasks = global.all_tasks().get();
		if let Some(all_tasks) = all_tasks {
			let missing: Vec<_> = all_tasks.iter().filter(|id| !processed_tasks.contains(*id)).collect();
			if missing.is_empty() {
				tracing::info!("received all task callbacks, stopping");
				break;
			} else {
				tracing::info!("missing {} task callbacks", missing.len());
			}
		}
	}

	Ok(())
}

async fn handle_success(
	global: &Arc<Global>,
	subject: Subject,
	metadata: HashMap<String, String>,
	event: event_callback::Success,
) -> anyhow::Result<()> {
	let input = event.input_metadata.context("missing input metadata")?;

	let animated = event.files.iter().any(|i| i.frame_count > 1);

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

	match subject {
		Subject::Emote(id) => {
			Emote::collection(global.target_db())
				.update_one(
					doc! {
						"_id": id,
					},
					doc! {
						"$set": {
							"animated": animated,
							"image_set.input.width": input.width,
							"image_set.input.height": input.height,
							"image_set.input.frame_count": input.frame_count,
							"image_set.outputs": to_bson(&outputs)?,
						},
					},
					None,
				)
				.await?;
		}
		Subject::ProfilePicture(id) => {
			let outputs = to_bson(&outputs)?;

			// https://www.mongodb.com/docs/manual/tutorial/update-documents-with-aggregation-pipeline
			let aggregation = vec![
				doc! {
					"$set": {
						"style.active_profile_picture.input.width": input.width,
						"style.active_profile_picture.input.height": input.height,
						"style.active_profile_picture.input.frame_count": input.frame_count,
						"style.active_profile_picture.outputs": outputs,
					},
				},
				// $push is not available in update pipelines
				// so we have to use $concatArrays to append to an array
				doc! {
					"$set": {
						"style.all_profile_pictures": {
							"$concatArrays": [
								"$style.all_profile_pictures",
								["$style.active_profile_picture"]
							],
						},
					},
				},
			];

			User::collection(global.target_db())
				.update_one(
					doc! {
						"_id": id,
					},
					aggregation,
					None,
				)
				.await?;
		}
		Subject::Paint(id) => {
			let layer_id: PaintLayerId = metadata.get("layer_id").context("missing layer_id")?.parse()?;

			Paint::collection(global.target_db())
				.update_one(
					doc! {
						"_id": id,
						"data.layers.id": layer_id,
					},
					doc! {
						"$set": {
							"data.layers.$.data.input.width": input.width,
							"data.layers.$.data.input.height": input.height,
							"data.layers.$.data.input.frame_count": input.frame_count,
							"data.layers.$.data.outputs": to_bson(&outputs)?,
						},
					},
					None,
				)
				.await?;
		}
		Subject::Badge(id) => {
			Badge::collection(global.target_db())
				.update_one(
					doc! {
						"_id": id,
					},
					doc! {
						"$set": {
							"image_set.input.width": input.width,
							"image_set.input.height": input.height,
							"image_set.input.frame_count": input.frame_count,
							"image_set.outputs": to_bson(&outputs)?,
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

async fn handle_abort(global: &Arc<Global>, subject: Subject, metadata: HashMap<String, String>) -> anyhow::Result<()> {
	match subject {
		Subject::Paint(id) => {
			let layer_id: PaintLayerId = metadata.get("layer_id").context("missing layer_id")?.parse()?;

			Paint::collection(global.target_db())
				.update_one(
					doc! {
						"_id": id,
						"data.layers": { "id": layer_id },
					},
					doc! {
						"$pull": {
							"data.layers": { "id": layer_id },
						},
					},
					None,
				)
				.await?;
		}
		Subject::Badge(id) => {
			Badge::collection(global.target_db())
				.delete_one(
					doc! {
						"_id": id,
					},
					None,
				)
				.await?;
		}
		Subject::Wildcard => anyhow::bail!("received event for wildcard subject"),
		_ => {}
	}

	Ok(())
}

async fn handle_fail(
	global: &Arc<Global>,
	subject: Subject,
	metadata: HashMap<String, String>,
	_event: event_callback::Fail,
) -> anyhow::Result<()> {
	handle_abort(global, subject, metadata).await?;

	// Notify user of failure with reason

	Ok(())
}

async fn handle_cancel(global: &Arc<Global>, subject: Subject, metadata: HashMap<String, String>) -> anyhow::Result<()> {
	handle_abort(global, subject, metadata).await?;

	// Notify user of cancellation

	Ok(())
}
