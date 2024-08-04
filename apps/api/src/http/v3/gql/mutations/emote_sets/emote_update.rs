use std::sync::Arc;

use hyper::StatusCode;
use mongodb::options::FindOneAndUpdateOptions;
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::event::EventEmoteSetData;
use shared::database::queries::{filter, update};
use shared::database::user::FullUser;
use shared::event::{EventPayload, EventPayloadData};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_update(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	actor: &FullUser,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
	name: Option<String>,
) -> TransactionResult<EmoteSet, ApiError> {
	let emote_set_emote = emote_set
		.emotes
		.iter()
		.find(|e| e.id == emote_id)
		.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found in set"))
		.map_err(TransactionError::custom)?;

	let new_name = if let Some(name) = name {
		name
	} else {
		let emote = global
			.emote_by_id_loader
			.load(emote_id)
			.await
			.map_err(|()| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
			.ok_or(TransactionError::custom(ApiError::new_const(
				StatusCode::NOT_FOUND,
				"emote not found",
			)))?;

		emote.default_name
	};

	if emote_set.emotes.iter().any(|e| e.alias == new_name) {
		return Err(TransactionError::custom(ApiError::new_const(
			StatusCode::CONFLICT,
			"emote name conflict",
		)));
	}

	let emote_set = tx
		.find_one_and_update(
			filter::filter! {
				EmoteSet {
					#[query(rename = "_id")]
					id: emote_set.id,
					#[query(flatten)]
					emotes: EmoteSetEmote {
						id: emote_id,
					}
				}
			},
			update::update! {
				#[query(set)]
				EmoteSet {
					#[query(flatten, index = "$")]
					emotes: EmoteSetEmote {
						alias: new_name.clone(),
					},
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

	tx.register_event(EventPayload {
		actor_id: Some(actor.id),
		data: EventPayloadData::EmoteSet {
			after: emote_set.clone(),
			data: EventEmoteSetData::RenameEmote {
				emote_id,
				old_alias: emote_set_emote.alias.clone(),
				new_alias: new_name.clone(),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(emote_set)
}
