use std::sync::Arc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::emote::{EmoteFlags, EmoteId};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestId, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::emote_set::{EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetKind};
use shared::database::queries::{filter, update};
use shared::database::stored_event::StoredEventEmoteModerationRequestData;
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

pub async fn emote_add(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	session: &Session,
	emote_set: &EmoteSet,
	id: EmoteId,
	name: Option<String>,
) -> TransactionResult<EmoteSet, ApiError> {
	let authed_user = session.user().map_err(TransactionError::Custom)?;

	if let Some(capacity) = emote_set.capacity {
		if emote_set.emotes.len() as i32 >= capacity {
			return Err(TransactionError::Custom(ApiError::bad_request(
				ApiErrorCode::BadRequest,
				"emote set is at capacity",
			)));
		}
	}

	let emote = global
		.emote_by_id_loader
		.load(id)
		.await
		.map_err(|()| {
			TransactionError::Custom(ApiError::internal_server_error(
				ApiErrorCode::LoadError,
				"failed to load emote",
			))
		})?
		.ok_or_else(|| TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found")))?;

	let alias = name.unwrap_or_else(|| emote.default_name.clone());

	if emote_set.emotes.iter().any(|e| e.alias == alias || e.id == id) {
		return Err(TransactionError::Custom(ApiError::conflict(
			ApiErrorCode::BadRequest,
			"this emote is already in the set or has a conflicting name",
		)));
	}

	if matches!(emote_set.kind, EmoteSetKind::Personal) {
		if emote.flags.contains(EmoteFlags::DeniedPersonal) {
			return Err(TransactionError::Custom(ApiError::bad_request(
				ApiErrorCode::BadRequest,
				"emote is not allowed in personal emote sets",
			)));
		} else if !emote.flags.contains(EmoteFlags::ApprovedPersonal) {
			let id = EmoteModerationRequestId::new();
			let country_code = global
				.geoip()
				.and_then(|g| g.lookup(session.ip()))
				.and_then(|c| c.iso_code)
				.map(|c| c.to_string());

			let request = tx
				.find_one_and_update(
					filter::filter! {
						EmoteModerationRequest {
							#[query(serde)]
							kind: EmoteModerationRequestKind::PersonalUse,
							emote_id: emote.id,
						}
					},
					update::update! {
						#[query(set_on_insert)]
						EmoteModerationRequest {
							#[query(rename = "_id")]
							id,
							user_id: authed_user.id,
							#[query(serde)]
							kind: EmoteModerationRequestKind::PersonalUse,
							reason: Some("User requested to add emote to a personal set".to_string()),
							emote_id: emote.id,
							#[query(serde)]
							status: EmoteModerationRequestStatus::Pending,
							country_code,
							assigned_to: vec![],
							priority: authed_user
								.computed
								.permissions
								.emote_moderation_request_priority
								.unwrap_or_default(),
							search_updated_at: None::<chrono::DateTime<chrono::Utc>>,
							updated_at: chrono::Utc::now(),
						},
					},
					FindOneAndUpdateOptions::builder().upsert(true).build(),
				)
				.await?
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::MutationError,
						"emote moderation failed to insert",
					))
				})?;

			if request.id == id {
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::EmoteModerationRequest {
						after: request,
						data: StoredEventEmoteModerationRequestData::Create,
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			let count = tx
				.count(
					filter::filter! {
						EmoteModerationRequest {
							#[query(serde)]
							kind: EmoteModerationRequestKind::PersonalUse,
							user_id: authed_user.id,
							#[query(serde)]
							status: EmoteModerationRequestStatus::Pending,
						}
					},
					None,
				)
				.await?;

			if count as i32
				> authed_user
					.computed
					.permissions
					.emote_moderation_request_limit
					.unwrap_or_default()
			{
				return Err(TransactionError::Custom(ApiError::bad_request(
					ApiErrorCode::LackingPrivileges,
					"too many pending moderation requests",
				)));
			}
		}
	}

	let emote_set_emote = EmoteSetEmote {
		id,
		added_by_id: Some(authed_user.id),
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

	let emote_set = tx
		.find_one_and_update(
			filter::filter! {
				EmoteSet {
					#[query(rename = "_id")]
					id: emote_set.id,
				}
			},
			update::update! {
				#[query(set)]
				EmoteSet {
					emotes_changed_since_reindex: true,
					updated_at: chrono::Utc::now(),
				},
				#[query(push)]
				EmoteSet {
					#[query(serde)]
					emotes: &emote_set_emote,
				},
			},
			FindOneAndUpdateOptions::builder()
				.return_document(ReturnDocument::After)
				.build(),
		)
		.await?
		.ok_or_else(|| TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "emote set not found")))?;

	if let Some(capacity) = emote_set.capacity {
		if emote_set.emotes.len() as i32 > capacity {
			return Err(TransactionError::Custom(ApiError::bad_request(
				ApiErrorCode::LoadError,
				"emote set is at capacity",
			)));
		}
	}

	let emote_owner = global.user_loader.load_fast(global, emote.owner_id).await.map_err(|_| {
		TransactionError::Custom(ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote owner"))
	})?;

	tx.register_event(InternalEvent {
		actor: Some(authed_user.clone()),
		session_id: session.user_session_id(),
		data: InternalEventData::EmoteSet {
			after: emote_set.clone(),
			data: InternalEventEmoteSetData::AddEmote {
				emote: Box::new(emote),
				emote_owner,
				emote_set_emote,
			},
		},
		timestamp: chrono::Utc::now(),
	})?;

	Ok(emote_set)
}
