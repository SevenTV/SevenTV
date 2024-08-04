use hyper::StatusCode;
use mongodb::options::FindOneAndUpdateOptions;
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::event::EventEmoteSetData;
use shared::database::queries::{filter, update};
use shared::database::user::FullUser;
use shared::event::{EventPayload, EventPayloadData};

use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_remove(
	mut tx: TransactionSession<'_, ApiError>,
	actor: &FullUser,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
) -> TransactionResult<EmoteSet, ApiError> {
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

	tx.register_event(EventPayload {
		actor_id: Some(actor.id),
		data: EventPayloadData::EmoteSet {
			after: emote_set.clone(),
			data: EventEmoteSetData::RemoveEmote { emote_id },
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(emote_set)
}
