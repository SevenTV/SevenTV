use std::sync::Arc;

use anyhow::Context;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use scuffle_image_processor_proto::event_callback;
use shared::database::badge::{Badge, BadgeId};
use shared::database::event::{EventBadgeData, ImageProcessorEvent};
use shared::database::queries::{filter, update};
use shared::event::{EventPayload, EventPayloadData};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	id: BadgeId,
	event: &event_callback::Success,
) -> TransactionResult<(), anyhow::Error> {
	let image_set = event_to_image_set(event).map_err(TransactionError::custom)?;

	let after = tx
		.find_one_and_update(
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
			FindOneAndUpdateOptions::builder()
				.return_document(ReturnDocument::After)
				.build(),
		)
		.await?
		.context("badge not found")
		.map_err(TransactionError::custom)?;

	tx.register_event(EventPayload {
		actor_id: None,
		data: EventPayloadData::Badge {
			after,
			data: EventBadgeData::Process {
				event: ImageProcessorEvent::Success(event.clone()),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
	event: &event_callback::Fail,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.badge_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::custom(anyhow::anyhow!("failed to query badge")))?
		.ok_or(TransactionError::custom(anyhow::anyhow!("failed to query badge")))?;

	tx.register_event(EventPayload {
		actor_id: None,
		data: EventPayloadData::Badge {
			after,
			data: EventBadgeData::Process {
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
	id: BadgeId,
	event: &event_callback::Start,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.badge_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::custom(anyhow::anyhow!("failed to query badge")))?
		.ok_or(TransactionError::custom(anyhow::anyhow!("failed to query badge")))?;

	tx.register_event(EventPayload {
		actor_id: None,
		data: EventPayloadData::Badge {
			after,
			data: EventBadgeData::Process {
				event: ImageProcessorEvent::Start(event.clone()),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_cancel(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
	event: &event_callback::Cancel,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.badge_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::custom(anyhow::anyhow!("failed to query badge")))?
		.ok_or(TransactionError::custom(anyhow::anyhow!("failed to query badge")))?;

	tx.register_event(EventPayload {
		actor_id: None,
		data: EventPayloadData::Badge {
			after,
			data: EventBadgeData::Process {
				event: ImageProcessorEvent::Cancel(event.clone()),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}
