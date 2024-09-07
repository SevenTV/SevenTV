use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use scuffle_image_processor_proto::event_callback;
use shared::database::emote::{Emote, EmoteFlags, EmoteId};
use shared::database::queries::{filter, update};
use shared::database::stored_event::{ImageProcessorEvent, StoredEventEmoteData};
use shared::event::{InternalEvent, InternalEventData};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	id: EmoteId,
	event: &event_callback::Success,
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

	tx.register_event(InternalEvent {
		actor: None,
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
