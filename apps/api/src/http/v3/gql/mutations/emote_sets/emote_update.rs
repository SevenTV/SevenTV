use std::sync::Arc;

use mongodb::options::FindOneAndUpdateOptions;
use shared::database::emote::{Emote, EmoteId};
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::queries::{filter, update};
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_update(
	_: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	session: &Session,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
	name: Option<String>,
) -> TransactionResult<EmoteSet, ApiError> {
	let authed_user = session.user().map_err(TransactionError::Custom)?;

	let old_emote_set_emote =
		emote_set.emotes.iter().find(|e| e.id == emote_id).ok_or_else(|| {
			TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set"))
		})?;

	let emote = tx
		.find_one(filter::filter! { Emote { #[query(rename = "_id")] id: emote_id } }, None)
		.await?
		.ok_or_else(|| TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found")))?;

	if emote.deleted || emote.merged.is_some() {
		return Err(TransactionError::Custom(ApiError::not_found(
			ApiErrorCode::BadRequest,
			"emote not found",
		)));
	}

	let new_name = name.unwrap_or(emote.default_name.clone());

	if emote_set.emotes.iter().any(|e| e.alias == new_name) {
		return Err(TransactionError::Custom(ApiError::conflict(
			ApiErrorCode::BadRequest,
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
		.ok_or_else(|| TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set")))?;

	let emote_set_emote =
		emote_set.emotes.iter().find(|e| e.id == emote_id).ok_or_else(|| {
			TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set"))
		})?;

	tx.register_event(InternalEvent {
		actor: Some(authed_user.clone()),
		session_id: session.user_session_id(),
		data: InternalEventData::EmoteSet {
			after: emote_set.clone(),
			data: InternalEventEmoteSetData::RenameEmote {
				emote: Box::new(emote),
				emote_set_emote: emote_set_emote.clone(),
				old_alias: old_emote_set_emote.alias.clone(),
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(emote_set)
}
