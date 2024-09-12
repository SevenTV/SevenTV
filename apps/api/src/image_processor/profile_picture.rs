use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use scuffle_image_processor_proto::event_callback;
use shared::database::queries::{filter, update};
use shared::database::stored_event::{ImageProcessorEvent, StoredEventUserProfilePictureData};
use shared::database::user::profile_picture::{UserProfilePicture, UserProfilePictureId};
use shared::database::user::{User, UserStyle};
use shared::event::{InternalEvent, InternalEventData};

use super::event_to_image_set;
use crate::global::Global;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

#[tracing::instrument(skip_all, fields(id = %id))]
pub async fn handle_success(
	mut tx: TransactionSession<'_, anyhow::Error>,
	_: &Arc<Global>,
	id: UserProfilePictureId,
	event: &event_callback::Success,
) -> TransactionResult<(), anyhow::Error> {
	let image_set = event_to_image_set(event).map_err(TransactionError::Custom)?;

	let Some(profile_picture) = tx
		.find_one_and_update(
			filter::filter! {
				UserProfilePicture {
					#[query(rename = "_id")]
					id: id,
				}
			},
			update::update! {
				#[query(set)]
				UserProfilePicture {
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
	else {
		tracing::warn!("could not find profile picture");
		return Ok(());
	};

	tx.find_one_and_update(
		filter::filter! {
			User {
				#[query(rename = "_id")]
				id: profile_picture.user_id,
				#[query(flatten)]
				style: UserStyle {
					pending_profile_picture: Some(profile_picture.id),
				}
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
				updated_at: chrono::Utc::now(),
			}
		},
		None,
	)
	.await?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::UserProfilePicture {
			after: profile_picture,
			data: StoredEventUserProfilePictureData::Process {
				event: ImageProcessorEvent::Success(Some(event.clone().into())),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}

pub async fn handle_fail(
	mut tx: TransactionSession<'_, anyhow::Error>,
	global: &Arc<Global>,
	id: UserProfilePictureId,
	event: &event_callback::Fail,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.user_profile_picture_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query profile picture")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("profile picture not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::UserProfilePicture {
			after,
			data: StoredEventUserProfilePictureData::Process {
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
	id: UserProfilePictureId,
	event: &event_callback::Start,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.user_profile_picture_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query profile picture")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("profile picture not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::UserProfilePicture {
			after,
			data: StoredEventUserProfilePictureData::Process {
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
	id: UserProfilePictureId,
	event: &event_callback::Cancel,
) -> TransactionResult<(), anyhow::Error> {
	let after = global
		.user_profile_picture_id_loader
		.load(id)
		.await
		.map_err(|_| TransactionError::Custom(anyhow::anyhow!("failed to query profile picture")))?
		.ok_or(TransactionError::Custom(anyhow::anyhow!("profile picture not found")))?;

	tx.register_event(InternalEvent {
		actor: None,
		session_id: None,
		data: InternalEventData::UserProfilePicture {
			after,
			data: StoredEventUserProfilePictureData::Process {
				event: ImageProcessorEvent::Cancel(*event),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(())
}
