use std::collections::HashMap;
use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use scuffle_image_processor_proto::event_callback;
use shared::database::emote::{Emote, EmoteFlags, EmoteId};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestId, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::queries::{filter, update};
use shared::database::stored_event::{ImageProcessorEvent, StoredEventEmoteData, StoredEventEmoteModerationRequestData};
use shared::event::{InternalEvent, InternalEventData};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: EmoteId,
	event: &event_callback::Success,
	metadata: &HashMap<String, String>,
) -> TransactionResult<(), anyhow::Error> {
	let image_set = event_to_image_set(event).map_err(TransactionError::custom)?;

	let bit_update = if image_set.outputs.iter().any(|i| i.frame_count > 1) {
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

	let aspect_ratio = image_set
		.input
		.aspect_ratio()
		.ok_or(TransactionError::custom(anyhow::anyhow!("failed to get aspect ratio")))?;

	let after = tx
		.find_one_and_update(
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
					image_set,
					aspect_ratio,
					updated_at: chrono::Utc::now(),
				},
				#[query(bit)]
				bit_update
			},
			FindOneAndUpdateOptions::builder()
				.return_document(ReturnDocument::After)
				.build(),
		)
		.await?
		.ok_or(TransactionError::custom(anyhow::anyhow!("emote not found")))?;

	let country_code = metadata
		.get("upload_ip")
		.and_then(|ip| ip.parse().ok())
		.and_then(|ip| global.geoip()?.lookup(ip))
		.and_then(|c| c.iso_code);

	let actor = global
		.user_loader
		.load(global, after.owner_id)
		.await
		.ok()
		.flatten()
		.ok_or_else(|| TransactionError::custom(anyhow::anyhow!("failed to load user")))?;

	let mod_request = EmoteModerationRequest {
		id: EmoteModerationRequestId::new(),
		emote_id: id,
		user_id: after.owner_id,
		priority: actor
			.computed
			.permissions
			.emote_moderation_request_priority
			.unwrap_or_default(),
		kind: EmoteModerationRequestKind::PublicListing,
		status: EmoteModerationRequestStatus::Pending,
		reason: Some("New upload".to_string()),
		updated_at: chrono::Utc::now(),
		search_updated_at: None,
		assigned_to: vec![],
		country_code: country_code.map(|c| c.to_string()),
	};

	tx.insert_one::<EmoteModerationRequest>(&mod_request, None).await?;

	tx.register_event(InternalEvent {
		actor: Some(actor.clone()),
		session_id: None,
		data: InternalEventData::EmoteModerationRequest {
			after: mod_request,
			data: StoredEventEmoteModerationRequestData::Create,
		},
		timestamp: chrono::Utc::now(),
	})?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Emote {
			after,
			data: StoredEventEmoteData::Process {
				event: ImageProcessorEvent::Success(Some(event.clone().into())),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Error>,
	_: &Arc<Global>,
	id: EmoteId,
	event: &event_callback::Fail,
) -> TransactionResult<(), anyhow::Error> {
	// TODO(troy): should we delete this emote?
	// Perhaps it would be benificial to create an audit log entry for why this
	// emote failed to process. and then set the state to failed stating this emote
	// was deleted because ... (reason)
	let after = tx
		.find_one_and_delete(
			filter::filter! {
				Emote {
					#[query(rename = "_id")]
					id,
				}
			},
			None,
		)
		.await?
		.ok_or(TransactionError::custom(anyhow::anyhow!("emote not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Emote {
			after,
			data: StoredEventEmoteData::Process {
				event: ImageProcessorEvent::Fail(event.clone()),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_start(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: EmoteId,
	event: &event_callback::Start,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.emote_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::custom(anyhow::anyhow!("failed to query emote")))?
		.ok_or(TransactionError::custom(anyhow::anyhow!("emote not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Emote {
			after,
			data: StoredEventEmoteData::Process {
				event: ImageProcessorEvent::Start(*event),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_cancel(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: EmoteId,
	event: &event_callback::Cancel,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.emote_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::custom(anyhow::anyhow!("failed to query emote")))?
		.ok_or(TransactionError::custom(anyhow::anyhow!("emote not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Emote {
			after,
			data: StoredEventEmoteData::Process {
				event: ImageProcessorEvent::Cancel(*event),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}
