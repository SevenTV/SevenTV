use std::sync::Arc;

use anyhow::Context;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use scuffle_image_processor_proto::event_callback;
use shared::cdn::PurgeRequest;
use shared::database::badge::{Badge, BadgeId};
use shared::database::queries::{filter, update};
use shared::database::stored_event::{ImageProcessorEvent, StoredEventBadgeData};
use shared::event::{InternalEvent, InternalEventData};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	_: &Arc<Global>,
	id: BadgeId,
	event: event_callback::Success,
) -> TransactionResult<PurgeRequest, anyhow::Error> {
	let image_set = event_to_image_set(event).map_err(TransactionError::Custom)?;

	let before = tx
		.find_one(
			filter::filter! {
			Badge {
				#[query(rename = "_id")]
				id: id,
			}
			},
			None,
		)
		.await?
		.context("badge not found")
		.map_err(TransactionError::Custom)?;

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
					search_updated_at: &None,
				}
			},
			FindOneAndUpdateOptions::builder()
				.return_document(ReturnDocument::After)
				.build(),
		)
		.await?
		.context("badge not found")
		.map_err(TransactionError::Custom)?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Badge {
			after,
			data: StoredEventBadgeData::Process {
				event: ImageProcessorEvent::Success,
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(PurgeRequest {
		files: before.image_set.outputs.iter().filter_map(|i| i.path.parse().ok()).collect(),
	})
}

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
	event: event_callback::Fail,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.badge_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query badge")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("failed to query badge")))?;

	let error = event.error.clone().unwrap_or_default();

	tracing::info!("badge {} failed: {:?}: {}", id, error.code(), error.message);

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Badge {
			after,
			data: StoredEventBadgeData::Process { event: event.into() },
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_start(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: BadgeId,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.badge_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query badge")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("failed to query badge")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Badge {
			after,
			data: StoredEventBadgeData::Process {
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
	id: BadgeId,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.badge_by_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query badge")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("failed to query badge")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::Badge {
			after,
			data: StoredEventBadgeData::Process {
				event: ImageProcessorEvent::Cancel,
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}
