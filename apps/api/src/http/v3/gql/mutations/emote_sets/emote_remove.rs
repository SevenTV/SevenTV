use hyper::StatusCode;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteSetData, AuditLogId};
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetId};
use shared::database::user::{FullUser, UserId};

use crate::http::error::ApiError;
use crate::queries::{FindOneAndUpdateQuery, TransactionError, TransactionResult, TransactionSession};

struct RemoveEmoteQuery {
	filter: RemoveEmoteFilter,
	update: RemoveEmoteUpdate,
	actor: UserId,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RemoveEmoteFilter {
	id: EmoteSetId,
	#[serde(rename = "emotes.id")]
	emote_id: EmoteId,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RemoveEmoteUpdate {
	#[serde(rename = "$pull")]
	pub pull: RemoveEmotePull,
	#[serde(rename = "$set")]
	pub set: RemoveEmoteSet,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RemoveEmotePull {
	pub emotes: EmotesId,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RemoveEmoteSet {
	pub emotes_changed_since_reindex: bool,
	#[serde(with = "shared::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EmotesId {
	pub id: EmoteId,
}

impl FindOneAndUpdateQuery for RemoveEmoteQuery {
	type Collection = EmoteSet;
	type Filter<'a> = &'a RemoveEmoteFilter;
	type Update<'a> = &'a RemoveEmoteUpdate;

	fn filter(&self) -> Self::Filter<'_> {
		&self.filter
	}

	fn update(&self) -> Self::Update<'_> {
		&self.update
	}

	fn options(&self) -> Option<mongodb::options::FindOneAndUpdateOptions> {
		Some(
			mongodb::options::FindOneAndUpdateOptions::builder()
				.return_document(mongodb::options::ReturnDocument::After)
				.build(),
		)
	}

	fn audit_logs(&self, resp: Option<&Self::Collection>) -> impl IntoIterator<Item = AuditLog> {
		resp.map(|_| AuditLog {
			id: AuditLogId::new(),
			actor_id: Some(self.actor),
			data: AuditLogData::EmoteSet {
				target_id: self.filter.id,
				data: AuditLogEmoteSetData::RemoveEmote {
					emote_id: self.filter.emote_id,
				},
			},
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		})
	}

	fn emit_events(&self, resp: Option<&Self::Collection>) -> impl IntoIterator<Item = ()> {
		todo!("emit event for for remove emote from set");
		None
	}
}

pub async fn emote_remove(
	mut session: TransactionSession<'_, Result<EmoteSet, ApiError>>,
	actor: &FullUser,
	emote_set: &EmoteSet,
	id: EmoteId,
) -> TransactionResult<Result<EmoteSet, ApiError>> {
	if !emote_set.emotes.iter().any(|e| e.id == id) {
		return Err(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote not found in set",
		)));
	}

	let emote_set = session
		.find_one_and_update(&RemoveEmoteQuery {
			filter: RemoveEmoteFilter {
				id: emote_set.id,
				emote_id: id,
			},
			update: RemoveEmoteUpdate {
				pull: RemoveEmotePull { emotes: EmotesId { id } },
				set: RemoveEmoteSet {
					emotes_changed_since_reindex: true,
					updated_at: chrono::Utc::now(),
				},
			},
			actor: actor.id,
		})
		.await?
		.ok_or(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote not found in set",
		)))?;

	// let active_emote = ActiveEmoteModel::from_db(
	//     active_emote.clone(),
	//     Some(EmotePartialModel::from_db(
	//         global
	//             .emote_by_id_loader
	//             .load(id.id())
	//             .await
	//             .map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
	//             .ok_or(ApiError::NOT_FOUND)?,
	//         None,
	//         &global.config.api.cdn_origin,
	//     )),
	// );
	// let active_emote = serde_json::to_value(active_emote).map_err(|e| {
	//     tracing::error!(error = %e, "failed to serialize emote");
	//     ApiError::INTERNAL_SERVER_ERROR
	// })?;

	// global
	//     .event_api
	//     .dispatch_event(
	//         EventType::UpdateEmoteSet,
	//         ChangeMap {
	//             id: self.emote_set.id.cast(),
	//             kind: ObjectKind::EmoteSet,
	//             actor: Some(UserPartialModel::from_db(
	//                 user.clone(),
	//                 None,
	//                 None,
	//                 &global.config.api.cdn_origin,
	//             )),
	//             pulled: vec![ChangeField {
	//                 key: "emotes".to_string(),
	//                 index: Some(index),
	//                 ty: ChangeFieldType::Object,
	//                 old_value: active_emote,
	//                 ..Default::default()
	//             }],
	//             ..Default::default()
	//         },
	//         self.emote_set.id,
	//     )
	//     .await
	//     .map_err(|e| {
	//         tracing::error!(error = %e, "failed to dispatch event");
	//         ApiError::INTERNAL_SERVER_ERROR
	//     })?;

	Ok(Ok(emote_set))
}
