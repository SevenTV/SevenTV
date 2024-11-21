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

pub async fn emote_remove(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	session: &Session,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
) -> TransactionResult<EmoteSet, ApiError> {
	let authed_user = session.user().map_err(TransactionError::Custom)?;

	let old_emotes: Vec<_> = emote_set
		.emotes
		.iter()
		.enumerate()
		.filter(|(_, e)| e.id == emote_id)
		.collect();

	if old_emotes.is_empty() {
		return Err(TransactionError::Custom(ApiError::not_found(
			ApiErrorCode::BadRequest,
			"emote not found in set",
		)));
	}

	let emote = tx
		.find_one(filter::filter! { Emote { #[query(rename = "_id")] id: emote_id } }, None)
		.await?;

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
					search_updated_at: &None,
				},
			},
			FindOneAndUpdateOptions::builder()
				.return_document(mongodb::options::ReturnDocument::After)
				.build(),
		)
		.await?
		.ok_or(TransactionError::Custom(ApiError::not_found(
			ApiErrorCode::BadRequest,
			"emote not found in set",
		)))?;

	let emote_owner = if let Some(e) = &emote {
		global.user_loader.load_fast(global, e.owner_id).await.map_err(|_| {
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::LoadError,
				"failed to load emote owner",
			))
		})?
	} else {
		None
	};

	for (index, old_emote_set_emote) in old_emotes {
		tx.register_event(InternalEvent {
			actor: Some(authed_user.clone()),
			session_id: session.user_session_id(),
			data: InternalEventData::EmoteSet {
				after: emote_set.clone(),
				data: InternalEventEmoteSetData::RemoveEmote {
					emote: emote.clone().map(Box::new),
					emote_owner: emote_owner.clone().map(Box::new),
					emote_set_emote: old_emote_set_emote.clone(),
					index,
				},
			},
			timestamp: chrono::Utc::now(),
		})?;
	}

	Ok(emote_set)
}
