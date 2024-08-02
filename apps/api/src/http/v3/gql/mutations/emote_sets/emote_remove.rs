use std::sync::Arc;

use hyper::StatusCode;
use itertools::Itertools;
use mongodb::options::FindOneAndUpdateOptions;
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::event::EventEmoteSetData;
use shared::database::queries::{filter, update};
use shared::database::user::FullUser;
use shared::event::{EventPayload, EventPayloadData};
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, EventType, ObjectKind};
use shared::old_types::UserPartialModel;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::rest::types::{ActiveEmoteModel, EmotePartialModel};
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_remove(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	actor: &FullUser,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
) -> TransactionResult<EmoteSet, ApiError> {
	let Some((index, emote)) = emote_set.emotes.iter().find_position(|e| e.id == emote_id) else {
		return Err(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote not found in set",
		)));
	};

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

	let active_emote = ActiveEmoteModel::from_db(
		emote.clone(),
		Some(EmotePartialModel::from_db(
			global
				.emote_by_id_loader
				.load(emote_id)
				.await
				.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
				.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?,
			None,
			&global.config.api.cdn_origin,
		)),
	);
	let active_emote = serde_json::to_value(active_emote)
		.map_err(|e| {
			tracing::error!(error = %e, "failed to serialize emote");
			ApiError::INTERNAL_SERVER_ERROR
		})
		.map_err(TransactionError::custom)?;

	global
		.event_api
		.dispatch_event(
			EventType::UpdateEmoteSet,
			ChangeMap {
				id: emote_set.id.cast(),
				kind: ObjectKind::EmoteSet,
				actor: Some(UserPartialModel::from_db(
					actor.clone(),
					None,
					None,
					&global.config.api.cdn_origin,
				)),
				pulled: vec![ChangeField {
					key: "emotes".to_string(),
					index: Some(index),
					ty: ChangeFieldType::Object,
					old_value: active_emote,
					..Default::default()
				}],
				..Default::default()
			},
			emote_set.id,
		)
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to dispatch event");
			ApiError::INTERNAL_SERVER_ERROR
		})
		.map_err(TransactionError::custom)?;

	Ok(emote_set)
}
