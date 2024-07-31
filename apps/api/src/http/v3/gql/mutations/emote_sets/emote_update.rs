use std::sync::Arc;

use hyper::StatusCode;
use itertools::Itertools;
use mongodb::options::FindOneAndUpdateOptions;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteSetData, AuditLogId};
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::queries::{filter, update};
use shared::database::user::FullUser;
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, EventType, ObjectKind};
use shared::old_types::UserPartialModel;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::rest::types::{ActiveEmoteModel, EmotePartialModel};
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_update(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	actor: &FullUser,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
	name: Option<String>,
) -> TransactionResult<EmoteSet, ApiError> {
	let (index, emote_set_emote) = emote_set
		.emotes
		.iter()
		.find_position(|e| e.id == emote_id)
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

	tx.insert_one(
		AuditLog {
			id: AuditLogId::new(),
			actor_id: Some(actor.id),
			data: AuditLogData::EmoteSet {
				target_id: emote_set.id,
				data: AuditLogEmoteSetData::RenameEmote {
					emote_id,
					old_name: emote_set_emote.alias.clone(),
					new_name: new_name.clone(),
				},
			},
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		},
		None,
	)
	.await?;

	let old_active_emote = ActiveEmoteModel::from_db(
		emote_set_emote.clone(),
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

	let mut new_active_emote = old_active_emote.clone();
	new_active_emote.name = new_name;

	let old_active_emote = serde_json::to_value(old_active_emote).map_err(|e| {
		tracing::error!(error = %e, "failed to serialize emote");
		TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
	})?;
	let new_active_emote = serde_json::to_value(new_active_emote).map_err(|e| {
		tracing::error!(error = %e, "failed to serialize emote");
		TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
	})?;

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
				updated: vec![ChangeField {
					key: "emotes".to_string(),
					index: Some(index),
					ty: ChangeFieldType::Object,
					old_value: old_active_emote,
					value: new_active_emote,
					..Default::default()
				}],
				..Default::default()
			},
			emote_set.id,
		)
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "failed to dispatch event");
			TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
		})?;
	Ok(emote_set)
}
