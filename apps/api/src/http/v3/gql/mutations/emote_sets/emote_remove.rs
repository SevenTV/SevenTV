use std::sync::Arc;

use hyper::StatusCode;
use itertools::Itertools;
use mongodb::options::FindOneAndUpdateOptions;
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::queries::{filter, update};
use shared::database::user::FullUser;
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_remove(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	actor: &FullUser,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
) -> TransactionResult<EmoteSet, ApiError> {
	let (index, old_emote_set_emote) = emote_set
		.emotes
		.iter()
		.find_position(|e| e.id == emote_id)
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found in set"))
		.map_err(TransactionError::custom)?;

	let emote = global
		.emote_by_id_loader
		.load(emote_id)
		.await
		.map_err(|()| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
		.ok_or(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote not found",
		)))?;

	let emote_set = tx
		.find_one_and_update(
			filter::filter! {
				EmoteSet {
					#[query(rename = "_id")]
					id: emote_set.id,
					#[query(flatten)]
					emotes: EmoteSetEmote {
						id: emote_id,
					},
				}
			},
			update::update! {
				#[query(pull)]
				EmoteSet {
					emotes: EmoteSetEmote {
						id: emote_id,
					},
				},
				#[query(set)]
				EmoteSet {
					emotes_changed_since_reindex: true,
					updated_at: chrono::Utc::now(),
				},
			},
			FindOneAndUpdateOptions::builder()
				.return_document(mongodb::options::ReturnDocument::After)
				.build(),
		)
		.await?
		.ok_or(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote not found in set",
		)))?;

	tx.register_event(InternalEvent {
		actor: Some(actor.clone()),
		data: InternalEventData::EmoteSet {
			after: emote_set.clone(),
			data: InternalEventEmoteSetData::RemoveEmote {
				emote,
				emote_set_emote: old_emote_set_emote.clone(),
				index,
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(emote_set)
}
