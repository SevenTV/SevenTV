use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_nats::jetstream::{consumer, stream};
use futures::StreamExt;
use mongodb::bson::{doc, to_bson};
use mongodb::options::ReturnDocument;
use prost::Message;
use scuffle_foundations::context::{self, ContextFutExt};
use scuffle_image_processor_proto::{event_callback, EventCallback};
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteData, AuditLogId};
use shared::database::badge::Badge;
use shared::database::emote::Emote;
use shared::database::image_set::Image;
use shared::database::paint::{Paint, PaintLayerId};
use shared::database::user::User;
use shared::database::{ClientExt, Collection};
use shared::event_api::types::{ChangeFieldBuilder, ChangeFieldType, ChangeMapBuilder, EventType, ObjectKind};
use shared::image_processor::Subject;
use shared::old_types::UserPartialModel;

use crate::global::Global;
use crate::http::v3::types::EmoteLifecycleModel;

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
		.filter_map(|i| i.path.map(|p| Image {
			path: p.path,
			mime: i.content_type,
			size: i.size as i64,
			width: i.width as i32,
			height: i.height as i32,
			frame_count: i.frame_count as i32,
		}))
		.collect();

	let input = event.input_metadata.and_then(|m| m.path.map(|p| {
		Image {
			path: p.path,
			mime: m.content_type,
			size: m.size as i64,
			width: m.width as i32,
			height: m.height as i32,
			frame_count: m.frame_count as i32,
		}
	}));

	let outputs = to_bson(&outputs)?;

	match subject {
		Subject::Emote(id) => {
			let emote = global
				.mongo()
				.with_transaction(|mut session| {
					let outputs = &outputs;

					async move {
						AuditLog::collection(&global.db())
							.insert_one(AuditLog {
								id: AuditLogId::new(),
								actor_id: None,
								data: AuditLogData::Emote {
									target_id: id,
									data: AuditLogEmoteData::Process,
								},
							})
							.session(session.get().as_mut())
							.await?;

						Emote::collection(&global.db())
							.find_one_and_update(
								doc! {
									"_id": id,
								},
								doc! {
									"$set": {
										"animated": animated,
										"image_set.input": to_bson(&input)?,
										"image_set.outputs": &outputs,
									},
								},
							)
							.session(session.get().as_mut())
							.return_document(ReturnDocument::After)
							.await
					}
				})
				.await?
				.context("emote not found")?;

			let global_config = global
				.global_config_loader()
				.load(())
				.await
				.ok()
				.flatten()
				.context("failed to load global config")?;

			let actor = global
				.user_loader()
				.load_fast(global, emote.owner_id)
				.await
				.ok()
				.context("failed to query owner")?
				.map(|u| UserPartialModel::from_db(u, &global_config, None, None, &global.config().api.cdn_origin));

			let change = ChangeFieldBuilder::default()
				.key("lifecycle")
				.ty(ChangeFieldType::Number)
				.old_value(EmoteLifecycleModel::Processing as i32)
				.value(EmoteLifecycleModel::Live as i32)
				.build()
				.unwrap();

			global
				.event_api()
				.dispatch_event(
					EventType::UpdateEmote,
					ChangeMapBuilder::default()
						.id(id.cast())
						.kind(ObjectKind::Emote)
						.actor(actor)
						.updated(vec![
							change.clone(),
							ChangeFieldBuilder::default()
								.key("versions")
								.index(0)
								.nested(true)
								.value(serde_json::to_value(vec![change])?)
								.build()
								.unwrap(),
						])
						.build()
						.unwrap(),
					id,
				)
				.await?;
		}
		Subject::ProfilePicture(id) => {
			// https://www.mongodb.com/docs/manual/tutorial/update-documents-with-aggregation-pipeline
			let aggregation = vec![
				doc! {
					"$set": {
						"style.active_profile_picture.input": to_bson(&input)?,
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

			User::collection(global.db())
				.update_one(
					doc! {
						"_id": id,
					},
					aggregation,
				)
				.await?;
		}
		Subject::Paint(id) => {
			let layer_id: PaintLayerId = metadata.get("layer_id").context("missing layer_id")?.parse()?;

			Paint::collection(global.db())
				.update_one(
					doc! {
						"_id": id,
						"data.layers.id": layer_id,
					},
					doc! {
						"$set": {
							"data.layers.$.data.input": to_bson(&input)?,
							"data.layers.$.data.outputs": to_bson(&outputs)?,
						},
					},
				)
				.await?;
		}
		Subject::Badge(id) => {
			Badge::collection(global.db())
				.update_one(
					doc! {
						"_id": id,
					},
					doc! {
						"$set": {
							"image_set.input": to_bson(&input)?,
							"image_set.outputs": to_bson(&outputs)?,
						},
					},
				)
				.await?;
		}
		Subject::Wildcard => anyhow::bail!("received event for wildcard subject"),
	}

	Ok(())
}

async fn handle_abort(global: &Arc<Global>, subject: Subject, metadata: HashMap<String, String>) -> anyhow::Result<()> {
	match subject {
		Subject::Emote(id) => {
			Emote::collection(global.db())
				.delete_one(doc! {
					"_id": id,
				})
				.await?;
		}
		Subject::ProfilePicture(id) => {
			User::collection(global.db())
				.update_one(doc! { "_id": id }, doc! { "style.active_profile_picture": null })
				.await?;
		}
		Subject::Paint(id) => {
			let layer_id: PaintLayerId = metadata.get("layer_id").context("missing layer_id")?.parse()?;

			Paint::collection(global.db())
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
				)
				.await?;
		}
		Subject::Badge(id) => {
			Badge::collection(global.db())
				.delete_one(doc! {
					"_id": id,
				})
				.await?;
		}
		Subject::Wildcard => anyhow::bail!("received event for wildcard subject"),
	}

	Ok(())
}

async fn handle_fail(
	global: &Arc<Global>,
	subject: Subject,
	metadata: HashMap<String, String>,
	event: event_callback::Fail,
) -> anyhow::Result<()> {
	if let Subject::Emote(id) = subject {
		let emote = Emote::collection(global.db())
			.find_one_and_delete(doc! {
				"_id": id,
			})
			.await?;

		if let Some(emote) = emote {
			let global_config = global
				.global_config_loader()
				.load(())
				.await
				.ok()
				.flatten()
				.context("failed to load global config")?;

			let actor = global
				.user_loader()
				.load_fast(global, emote.owner_id)
				.await
				.ok()
				.context("failed to query owner")?
				.map(|u| UserPartialModel::from_db(u, &global_config, None, None, &global.config().api.cdn_origin));

			let change = ChangeFieldBuilder::default()
				.key("lifecycle")
				.ty(ChangeFieldType::Number)
				.old_value(EmoteLifecycleModel::Processing as i32)
				.value(EmoteLifecycleModel::Failed as i32)
				.build()
				.unwrap();

			global
				.event_api()
				.dispatch_event(
					EventType::UpdateEmote,
					ChangeMapBuilder::default()
						.id(id.cast())
						.kind(ObjectKind::Emote)
						.actor(actor)
						.updated(vec![
							change.clone(),
							ChangeFieldBuilder::default()
								.key("versions")
								.index(0)
								.nested(true)
								.value(serde_json::to_value(vec![change])?)
								.build()
								.unwrap(),
							ChangeFieldBuilder::default()
								.key("versions")
								.index(0)
								.nested(true)
								.value(serde_json::to_value(vec![ChangeFieldBuilder::default()
									.key("error")
									.ty(ChangeFieldType::String)
									.value(event.error.map(|e| e.message).unwrap_or_default())
									.build()
									.unwrap()])?)
								.build()
								.unwrap(),
						])
						.build()
						.unwrap(),
					id,
				)
				.await?;
		}
	} else {
		handle_abort(global, subject, metadata).await?;
	}

	// Notify user of failure with reason

	Ok(())
}

async fn handle_cancel(global: &Arc<Global>, subject: Subject, metadata: HashMap<String, String>) -> anyhow::Result<()> {
	handle_abort(global, subject, metadata).await?;

	// Notify user of cancellation

	Ok(())
}
