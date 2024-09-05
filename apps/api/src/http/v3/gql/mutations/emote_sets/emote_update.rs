use std::sync::Arc;

use hyper::StatusCode;
use mongodb::options::FindOneAndUpdateOptions;
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::queries::{filter, update};
use shared::database::user::session::UserSessionId;
use shared::database::user::FullUser;
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_update(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	auth_session_id: Option<UserSessionId>,
	authed_user: &FullUser,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
	name: Option<String>,
) -> TransactionResult<EmoteSet, ApiError> {
	let old_emote_set_emote = emote_set
		.emotes
		.iter()
		.find(|e| e.id == emote_id)
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

	let new_name = name.unwrap_or(emote.default_name.clone());

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

	let emote_set_emote = emote_set
		.emotes
		.iter()
		.find(|e| e.id == emote_id)
		.ok_or(ApiError::new_const(
			StatusCode::INTERNAL_SERVER_ERROR,
			"emote not found in set",
		))
		.map_err(TransactionError::custom)?;

	tx.register_event(InternalEvent {
		actor: Some(authed_user.clone()),
		session_id: auth_session_id,
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
