use std::sync::Arc;

use hyper::StatusCode;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteSetData, AuditLogId};
use shared::database::emote::{EmoteFlags, EmoteId};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestId, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::emote_set::{EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetId, EmoteSetKind};
use shared::database::user::{FullUser, UserId};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::queries::{
	CountQuery, FindOneAndUpdateQuery, MongoSetOnInsert, TransactionError, TransactionResult, TransactionSession,
};

struct CreateEmoteModerationRequestQuery {
	filter: CreateEmoteModerationRequestFilter,
	update: EmoteModerationRequest,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateEmoteModerationRequestFilter {
	kind: EmoteModerationRequestKind,
	emote_id: EmoteId,
}

impl FindOneAndUpdateQuery for CreateEmoteModerationRequestQuery {
	type Collection = EmoteModerationRequest;
	type Filter<'a> = &'a CreateEmoteModerationRequestFilter;
	type Update<'a> = MongoSetOnInsert<&'a EmoteModerationRequest>;

	fn filter(&self) -> Self::Filter<'_> {
		&self.filter
	}

	fn update(&self) -> Self::Update<'_> {
		MongoSetOnInsert {
			set_on_insert: &self.update,
		}
	}

	fn options(&self) -> Option<mongodb::options::FindOneAndUpdateOptions> {
		Some(
			mongodb::options::FindOneAndUpdateOptions::builder()
				.upsert(true)
				.return_document(mongodb::options::ReturnDocument::After)
				.build(),
		)
	}

	fn audit_logs(
		&self,
		resp: Option<&Self::Collection>,
	) -> impl IntoIterator<Item = shared::database::audit_log::AuditLog> {
		todo!("add audit log for emote moderation request");
		None
	}

	fn emit_events(&self, resp: Option<&Self::Collection>) -> impl IntoIterator<Item = ()> {
		todo!("emit event for emote moderation request");
		None
	}
}

pub struct CountEmoteModerationRequestQuery {
	filter: CountEmoteModerationRequestFilter,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CountEmoteModerationRequestFilter {
	kind: EmoteModerationRequestKind,
	user_id: UserId,
	status: EmoteModerationRequestStatus,
}

impl CountQuery for CountEmoteModerationRequestQuery {
	type Collection = EmoteModerationRequest;
	type Filter<'a> = &'a CountEmoteModerationRequestFilter;

	fn filter(&self) -> Self::Filter<'_> {
		&self.filter
	}
}

pub struct UpdateEmoteSetQuery {
	filter: UpdateEmoteSetFilter,
	update: UpdateEmoteSetUpdate,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateEmoteSetFilter {
	id: EmoteSetId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateEmoteSetUpdate {
	#[serde(rename = "$set")]
	pub set: UpdateEmoteSetUpdateSet,
	#[serde(rename = "$push")]
	pub push: UpdateEmoteSetUpdatePush,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateEmoteSetUpdateSet {
	pub emotes_changed_since_reindex: bool,
	#[serde(with = "shared::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateEmoteSetUpdatePush {
	pub emotes: EmoteSetEmote,
}

impl FindOneAndUpdateQuery for UpdateEmoteSetQuery {
	type Collection = EmoteSet;
	type Filter<'a> = &'a UpdateEmoteSetFilter;
	type Update<'a> = &'a UpdateEmoteSetUpdate;

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
			actor_id: self.update.push.emotes.added_by_id,
			data: AuditLogData::EmoteSet {
				target_id: self.filter.id,
				data: AuditLogEmoteSetData::AddEmote {
					emote_id: self.update.push.emotes.id,
					alias: self.update.push.emotes.alias.clone(),
				},
			},
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		})
	}

	fn emit_events(&self, resp: Option<&Self::Collection>) -> impl IntoIterator<Item = ()> {
		todo!("emit event for add emote to emote set");
		None
	}
}

pub async fn emote_add(
	global: &Arc<Global>,
	mut session: TransactionSession<'_, Result<EmoteSet, ApiError>>,
	actor: &FullUser,
	target: &FullUser,
	emote_set: &EmoteSet,
	id: EmoteId,
	name: Option<String>,
) -> TransactionResult<Result<EmoteSet, ApiError>> {
	if let Some(capacity) = emote_set.capacity {
		if emote_set.emotes.len() as i32 >= capacity {
			return Err(TransactionError::custom(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"emote set is at capacity",
			)));
		}
	}

	let emote = global
		.emote_by_id_loader
		.load(id)
		.await
		.map_err(|()| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
		.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

	let alias = name.unwrap_or_else(|| emote.default_name.clone());

	if emote_set.emotes.iter().any(|e| e.alias == alias || e.id == id) {
		return Err(TransactionError::custom(ApiError::new_const(
			StatusCode::CONFLICT,
			"this emote is already in the set or has a conflicting name",
		)));
	}

	if matches!(emote_set.kind, EmoteSetKind::Personal) {
		if emote.flags.contains(EmoteFlags::DeniedPersonal) {
			return Err(TransactionError::custom(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"emote is not allowed in personal emote sets",
			)));
		} else if !emote.flags.contains(EmoteFlags::ApprovedPersonal) {
			let inserted_id = EmoteModerationRequestId::new();

			let request = session
				.find_one_and_update(&CreateEmoteModerationRequestQuery {
					filter: CreateEmoteModerationRequestFilter {
						kind: EmoteModerationRequestKind::PersonalUse,
						emote_id: emote.id,
					},
					update: EmoteModerationRequest {
						id: inserted_id,
						emote_id: emote.id,
						country_code: None,
						kind: EmoteModerationRequestKind::PersonalUse,
						assigned_to: vec![],
						priority: actor
							.computed
							.permissions
							.emote_moderation_request_priority
							.unwrap_or_default(),
						reason: Some("User requested to add emote to a personal set".to_string()),
						status: EmoteModerationRequestStatus::Pending,
						user_id: actor.id,
						search_updated_at: None,
						updated_at: chrono::Utc::now(),
					},
				})
				.await?
				.ok_or(TransactionError::custom(ApiError::new_const(
					StatusCode::NOT_FOUND,
					"emote moderation request not found",
				)))?;

			// We only care to check if this is the result we just inserted
			if request.id == inserted_id {
				let count = session
					.count(&CountEmoteModerationRequestQuery {
						filter: CountEmoteModerationRequestFilter {
							kind: EmoteModerationRequestKind::PersonalUse,
							user_id: target.id,
							status: EmoteModerationRequestStatus::Pending,
						},
					})
					.await?;

				if count as i32 > target.computed.permissions.emote_moderation_request_limit.unwrap_or_default() {
					return Err(TransactionError::custom(ApiError::new_const(
						StatusCode::BAD_REQUEST,
						"too many pending moderation requests",
					)));
				}
			}
		}
	}

	let emote_set_emote = EmoteSetEmote {
		id,
		added_by_id: Some(actor.id),
		alias: alias.clone(),
		flags: {
			if emote.flags.contains(EmoteFlags::DefaultZeroWidth) {
				EmoteSetEmoteFlag::ZeroWidth
			} else {
				EmoteSetEmoteFlag::default()
			}
		},
		added_at: chrono::Utc::now(),
		origin_set_id: None,
	};

	let emote_set = session
		.find_one_and_update(&UpdateEmoteSetQuery {
			filter: UpdateEmoteSetFilter { id: emote_set.id },
			update: UpdateEmoteSetUpdate {
				set: UpdateEmoteSetUpdateSet {
					emotes_changed_since_reindex: true,
					updated_at: chrono::Utc::now(),
				},
				push: UpdateEmoteSetUpdatePush { emotes: emote_set_emote },
			},
		})
		.await?
		.ok_or(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote set not found",
		)))?;

	if let Some(capacity) = emote_set.capacity {
		if emote_set.emotes.len() as i32 > capacity {
			return Err(TransactionError::custom(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"emote set is at capacity",
			)));
		}
	}

	// let active_emote = ActiveEmoteModel::from_db(
	//     emote_set_emote,
	//     Some(EmotePartialModel::from_db(emote, None,
	// &global.config.api.cdn_origin)), );
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
	//             pushed: vec![ChangeField {
	//                 key: "emotes".to_string(),
	//                 index: Some(emote_set.emotes.len()),
	//                 ty: ChangeFieldType::Object,
	//                 value: active_emote,
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
