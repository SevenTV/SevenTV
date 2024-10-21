use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use scuffle_image_processor_proto::event_callback;
use shared::database::paint::{Paint, PaintData, PaintId, PaintLayer, PaintLayerId, PaintLayerType};
use shared::database::queries::{filter, update};
use shared::database::stored_event::{ImageProcessorEvent, StoredEventPaintData};
use shared::event::{InternalEvent, InternalEventData};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	_: &Arc<Global>,
	id: PaintId,
	layer_id: PaintLayerId,
	event: event_callback::Success,
) -> TransactionResult<(), anyhow::Error> {
	let image_set = event_to_image_set(event).map_err(TransactionError::Custom)?;

	let after = tx
		.find_one_and_update(
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
							ty: PaintLayerType::Image(image_set),
						}
					},
					updated_at: chrono::Utc::now(),
					search_updated_at: &None,
				}
			},
			FindOneAndUpdateOptions::builder()
				.return_document(ReturnDocument::After)
				.build(),
		)
		.await?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("paint not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Paint {
			after,
			data: StoredEventPaintData::Process {
				event: ImageProcessorEvent::Success,
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: PaintId,
	event: event_callback::Fail,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.paint_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query paint")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("paint not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Paint {
			after,
			data: StoredEventPaintData::Process { event: event.into() },
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_start(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: PaintId,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.paint_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query paint")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("paint not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Paint {
			after,
			data: StoredEventPaintData::Process {
				event: ImageProcessorEvent::Start,
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_cancel(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: PaintId,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.paint_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query paint")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("paint not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Paint {
			after,
			data: StoredEventPaintData::Process {
				event: ImageProcessorEvent::Cancel,
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}
