use std::sync::Arc;

use scuffle_image_processor_proto::event_callback;
use shared::database::badge::{Badge, BadgeId};
use shared::database::queries::{filter, update};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
	event: &event_callback::Success,
) -> TransactionResult<(), anyhow::Error> {
	let image_set = event_to_image_set(event).map_err(TransactionError::custom)?;

	tx.update_one(
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
				image_set,
				updated_at: chrono::Utc::now(),
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

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
	event: &event_callback::Fail,
) -> TransactionResult<(), anyhow::Error> {
	todo!()
}

pub async fn handle_start(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
	event: &event_callback::Start,
) -> TransactionResult<(), anyhow::Error> {
	todo!()
}

pub async fn handle_cancel(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
	event: &event_callback::Cancel,
) -> TransactionResult<(), anyhow::Error> {
	todo!()
}
