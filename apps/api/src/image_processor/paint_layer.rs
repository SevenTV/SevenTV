use std::sync::Arc;

use scuffle_image_processor_proto::event_callback;
use shared::database::paint::{Paint, PaintData, PaintId, PaintLayer, PaintLayerId, PaintLayerType};
use shared::database::queries::{filter, update};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Result<()>>,
	global: &Arc<Global>,
	id: PaintId,
	layer_id: PaintLayerId,
	event: &event_callback::Success,
) -> TransactionResult<anyhow::Result<()>> {
	let image_set = event_to_image_set(event).map_err(TransactionError::custom)?;

	tx.update_one(
		filter::filter! {
			Paint {
				#[query(rename = "_id")]
				id: id,
				#[query(flatten)]
				data: PaintData {
					#[query(flatten)]
					layers: PaintLayer {
						#[query(rename = "_id")]
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
			}
		},
		None,
	)
	.await?;

	// TODO(lennart): audit log for this event?
	// TODO(lennart): event emission
	tx.register_event(());

	Ok(Ok(()))
}

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Result<()>>,
	global: &Arc<Global>,
	id: PaintId,
	layer_id: PaintLayerId,
	event: &event_callback::Fail,
) -> TransactionResult<anyhow::Result<()>> {
	todo!()
}

pub async fn handle_start(
	mut tx: TransactionSession<'_, anyhow::Result<()>>,
	global: &Arc<Global>,
	id: PaintId,
	layer_id: PaintLayerId,
	event: &event_callback::Start,
) -> TransactionResult<anyhow::Result<()>> {
	todo!()
}

pub async fn handle_cancel(
	mut tx: TransactionSession<'_, anyhow::Result<()>>,
	global: &Arc<Global>,
	id: PaintId,
	layer_id: PaintLayerId,
	event: &event_callback::Cancel,
) -> TransactionResult<anyhow::Result<()>> {
	todo!()
}
