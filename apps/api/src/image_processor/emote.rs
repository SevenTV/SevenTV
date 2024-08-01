use std::sync::Arc;

use scuffle_image_processor_proto::event_callback;
use shared::database::event::{Event, EventData, EventEmoteData, EventId};
use shared::database::emote::{Emote, EmoteFlags, EmoteId};
use shared::database::queries::{filter, update};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
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

	tx.update_one(
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
		None,
	)
	.await?;

	tx.insert_one(
		Event {
			id: EventId::new(),
			actor_id: None,
			data: EventData::Emote {
				target_id: id,
				data: EventEmoteData::Process,
			},
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		},
		None,
	)
	.await?;

	// TODO(lennart): when we design the new event system we can update this
	// placeholder to be a real event but this is how it will look we would pass a
	// struct here and when the transaction is committed the tx handler will emit
	// all events that were registered during this transaction.
	tx.register_event(());

	Ok(())
}

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: EmoteId,
	event: &event_callback::Fail,
) -> TransactionResult<(), anyhow::Error> {
	// TODO(troy): should we delete this emote?
	// Perhaps it would be benificial to create an audit log entry for why this
	// emote failed to process. and then set the state to failed stating this emote
	// was deleted because ... (reason)
	tx.delete_one(
		filter::filter! {
			Emote {
				#[query(rename = "_id")]
				id,
			}
		},
		None,
	)
	.await?;

	// TODO(lennart): audit log for this event?
	// TODO(lennart): event emission
	tx.register_event(());

	Ok(())
}

pub async fn handle_start(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: EmoteId,
	event: &event_callback::Start,
) -> TransactionResult<(), anyhow::Error> {
	// TODO(lennart): do we do anything here?
	Ok(())
}

pub async fn handle_cancel(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: EmoteId,
	event: &event_callback::Cancel,
) -> TransactionResult<(), anyhow::Error> {
	// TODO(lennart): do we do anything here?
	Ok(())
}
