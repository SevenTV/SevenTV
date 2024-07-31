use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream::{consumer, stream};
use futures::StreamExt;
use mongodb::bson::doc;
use prost::Message;
use scuffle_foundations::context::{self, ContextFutExt};
use scuffle_image_processor_proto::{event_callback, EventCallback};
use shared::database::badge::Badge;
use shared::database::emote::{Emote, EmoteFlags};
use shared::database::image_set::{Image, ImageSet, ImageSetInput};
use shared::database::paint::{Paint, PaintData, PaintLayer, PaintLayerId, PaintLayerType};
use shared::database::queries::{filter, update};
use shared::database::user::profile_picture::UserProfilePicture;
use shared::database::user::{User, UserStyle};
use shared::database::MongoCollection;
use shared::image_processor::Subject;

use crate::global::Global;

const JETSTREAM_NAME: &str = "image-processor-callback";
const JETSTREAM_CONSUMER_NAME: &str = "image-processor-callback-consumer";

pub async fn run(global: Arc<Global>) -> Result<(), anyhow::Error> {
	if !global.config().should_run_cosmetics() {
		tracing::info!("image processor callback job is disabled");
		return Ok(());
	}

	let config = &global.config().image_processor;

	let subject = Subject::wildcard(&config.event_queue_topic_prefix);

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
		let subject = match Subject::from_string(&message.subject, &global.config().image_processor.event_queue_topic_prefix)
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
			path: i.path.map(|p| p.path).unwrap_or_default(),
			mime: i.content_type,
			size: i.size as i64,
			width: i.width as i32,
			height: i.height as i32,
			frame_count: i.frame_count as i32,
		})
		.collect();

	match subject {
		Subject::Emote(id) => {
			let bit_update = if animated {
				Some(update::update! {
					#[query(bit)]
					Emote {
						#[query(bit = "or")]
						flags: EmoteFlags::Animated
					}
				})
			} else {
				None
			};

			Emote::collection(global.target_db())
				.update_one(
					filter::filter! {
						Emote {
							#[query(rename = "_id")]
							id: id,
						}
					},
					update::update! {
						#[query(set)]
						Emote {
							#[query(serde)]
							image_set: ImageSet {
								input: ImageSetInput::Image(Image {
									frame_count: input.frame_count as i32,
									width: input.width as i32,
									height: input.height as i32,
									path: input.path.map(|p| p.path).unwrap_or_default(),
									mime: input.content_type,
									size: input.size as i64,
								}),
								outputs,
							},
						},
						#[query(bit)]
						bit_update
					},
				)
				.await?;
		}
		Subject::ProfilePicture(id) => {
			let profile_picture = UserProfilePicture::collection(global.target_db())
				.find_one_and_update(
					filter::filter! {
						UserProfilePicture {
							#[query(rename = "_id")]
							id,
						}
					},
					update::update! {
						#[query(set)]
						UserProfilePicture {
							#[query(serde)]
							image_set: ImageSet {
								input: ImageSetInput::Image(Image {
									frame_count: input.frame_count as i32,
									width: input.width as i32,
									height: input.height as i32,
									path: input.path.map(|p| p.path).unwrap_or_default(),
									mime: input.content_type,
									size: input.size as i64,
								}),
								outputs,
							},
						}
					},
				)
				.await?
				.ok_or_else(|| anyhow::anyhow!("failed to update user profile picture, no matching pfp found"))?;

			User::collection(global.target_db())
				.update_one(
					filter::filter! {
						User {
							id: profile_picture.user_id,
							#[query(flatten)]
							style: UserStyle {
								pending_profile_picture: Some(profile_picture.id),
							},
						}
					},
					update::update! {
						#[query(set)]
						User {
							#[query(flatten)]
							style: UserStyle {
								active_profile_picture: Some(profile_picture.id),
								pending_profile_picture: &None,
							},
						}
					},
				)
				.await?;
		}
		Subject::PaintLayer(id, layer_id) => {
			Paint::collection(global.target_db())
				.update_one(
					filter::filter! {
						Paint {
							#[query(rename = "_id")]
							id: id,
							#[query(flatten)]
							data: PaintData {
								#[query(flatten)]
								layers: PaintLayer {
									id: layer_id,
								}
							}
						}
					},
					update::update! {
						#[query(set)]
						Paint {
							#[query(flatten)]
							data: PaintData {
								#[query(index = "$", flatten)]
								layers: PaintLayer {
									#[query(serde)]
									ty: PaintLayerType::Image(ImageSet {
										input: ImageSetInput::Image(Image {
											frame_count: input.frame_count as i32,
											width: input.width as i32,
											height: input.height as i32,
											path: input.path.map(|p| p.path).unwrap_or_default(),
											mime: input.content_type,
											size: input.size as i64,
										}),
										outputs,
									}),
								}
							}
						}
					},
				)
				.await?;
		}
		Subject::Badge(id) => {
			Badge::collection(global.target_db())
				.update_one(
					filter::filter! {
						Badge {
							#[query(rename = "_id")]
							id: id,
						}
					},
					update::update! {
						#[query(set)]
						Badge {
							#[query(serde)]
							image_set: ImageSet {
								input: ImageSetInput::Image(Image {
									frame_count: input.frame_count as i32,
									width: input.width as i32,
									height: input.height as i32,
									path: input.path.map(|p| p.path).unwrap_or_default(),
									mime: input.content_type,
									size: input.size as i64,
								}),
								outputs,
							},
						}
					},
				)
				.await?;
		}
	}

	Ok(())
}

async fn handle_abort(global: &Arc<Global>, subject: Subject, metadata: HashMap<String, String>) -> anyhow::Result<()> {
	match subject {
		Subject::PaintLayer(id, layer_id) => {
			Paint::collection(global.target_db())
				.update_one(
					filter::filter! {
						Paint {
							#[query(rename = "_id")]
							id: id,
							#[query(flatten)]
							data: PaintData {
								#[query(flatten)]
								layers: PaintLayer {
									id: layer_id,
								}
							}
						}
					},
					update::update! {
						#[query(pull)]
						Paint {
							#[query(flatten)]
							data: PaintData {
								layers: PaintLayer {
									id: layer_id,
								}
							}
						},
					},
				)
				.await?;
		}
		Subject::Badge(id) => {
			Badge::collection(global.target_db())
				.delete_one(filter::filter! {
					Badge {
						#[query(rename = "_id")]
						id: id,
					}
				})
				.await?;
		}
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
