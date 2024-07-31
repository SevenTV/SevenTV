use std::sync::Arc;

use hyper::StatusCode;
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet, EmoteSetEmote};
use shared::database::queries::{filter, update};
use shared::database::user::FullUser;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::transactions::{TransactionError, TransactionResult, TransactionSession};

// struct UpdateEmoteQuery {
// 	filter: UpdateEmoteFilter,
// 	update: UpdateEmoteUpdate,
// 	actor: UserId,
// 	old_name: String,
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// struct UpdateEmoteFilter {
// 	id: EmoteSetId,
// 	#[serde(rename = "emotes.id")]
// 	emote_id: EmoteId,
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// struct UpdateEmoteUpdate {
// 	#[serde(rename = "emotes.$.alias")]
// 	alias: String,
// 	emotes_changed_since_reindex: bool,
// 	#[serde(with = "shared::database::serde")]
// 	updated_at: chrono::DateTime<chrono::Utc>,
// }

// impl FindOneAndUpdateQuery for UpdateEmoteQuery {
// 	type Collection = EmoteSet;
// 	type Filter<'a> = &'a UpdateEmoteFilter;
// 	type Update<'a> = MongoSet<&'a UpdateEmoteUpdate>;

// 	fn filter(&self) -> Self::Filter<'_> {
// 		&self.filter
// 	}

// 	fn update(&self) -> Self::Update<'_> {
// 		MongoSet { set: &self.update }
// 	}

// 	fn options(&self) -> Option<mongodb::options::FindOneAndUpdateOptions> {
// 		Some(
// 			mongodb::options::FindOneAndUpdateOptions::builder()
// 				.return_document(mongodb::options::ReturnDocument::After)
// 				.build(),
// 		)
// 	}

// 	fn audit_logs(&self, resp: Option<&Self::Collection>) -> impl IntoIterator<Item = AuditLog> {
// 		resp.map(|_| AuditLog {
// 			id: AuditLogId::new(),
// 			actor_id: Some(self.actor),
// 			data: AuditLogData::EmoteSet {
// 				target_id: self.filter.id,
// 				data: AuditLogEmoteSetData::RenameEmote {
// 					emote_id: self.filter.emote_id,
// 					old_name: self.old_name.clone(),
// 					new_name: self.update.alias.clone(),
// 				},
// 			},
// 			updated_at: chrono::Utc::now(),
// 			search_updated_at: None,
// 		})
// 	}

// 	fn emit_events(&self, resp: Option<&Self::Collection>) -> impl IntoIterator<Item = ()> {
// 		todo!("emit event for emote moderation request");
// 		None
// 	}
// }

pub async fn emote_update(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	actor: &FullUser,
	emote_set: &EmoteSet,
	emote_id: EmoteId,
	name: Option<String>,
) -> TransactionResult<EmoteSet, ApiError> {
	let Some(emote) = emote_set.emotes.iter().find(|e| e.id == emote_id) else {
		return Err(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote not found in set",
		)));
	};

	let name = if let Some(name) = name {
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

	if emote_set.emotes.iter().any(|e| e.alias == name) {
		return Err(TransactionError::custom(ApiError::new_const(
			StatusCode::CONFLICT,
			"emote already has this name",
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
						alias: name,
					},
					emotes_changed_since_reindex: true,
					updated_at: chrono::Utc::now(),
				},
			},
			None,
		)
		.await?
		.ok_or(TransactionError::custom(ApiError::new_const(
			StatusCode::NOT_FOUND,
			"emote not found in set",
		)))?;

	Ok(emote_set)
}

// let (index, emote_set_emote) = self
// 						.emote_set
// 						.emotes
// 						.iter()
// 						.find_position(|e| e.id == id.id())
// 						.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found in
// set"))?;

// 					let name = if let Some(name) = name {
// 						name
// 					} else {
// 						let emote = global
// 							.emote_by_id_loader
// 							.load(id.id())
// 							.await
// 							.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
// 							.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not
// found"))?;

// 						emote.default_name
// 					};

// 					if emote_set_emote.alias == name {
// 						return Err(ApiError::new_const(StatusCode::CONFLICT, "emote already has
// this name")); 					}

// 					self.emote_set
// 						.emotes
// 						.iter()
// 						.find(|e| e.alias == name)
// 						.ok_or(ApiError::new_const(
// 							StatusCode::CONFLICT,
// 							"emote with this name already exists in set",
// 						))?;

// 					let emote_set = DbEmoteSet::collection(&global.db)
// 						.find_one_and_update(
// 							doc! {
// 								"_id": self.emote_set.id,
// 								"emotes.id": id.0,
// 							},
// 							doc! {
// 								"$set": {
// 									"emotes.$.alias": &name,
// 									"emotes_changed_since_reindex": true,
// 									"updated_at": Some(bson::DateTime::from(chrono::Utc::now())),
// 								},
// 							},
// 						)
// 						.session(&mut session)
// 						.return_document(ReturnDocument::After)
// 						.await
// 						.map_err(|e| {
// 							tracing::error!(error = %e, "failed to update emote in set");
// 							ApiError::INTERNAL_SERVER_ERROR
// 						})?
// 						.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote set not
// found"))?;

// 					AuditLog::collection(&global.db)
// 						.insert_one(AuditLog {
// 							id: AuditLogId::new(),
// 							actor_id: Some(auth_session.user_id()),
// 							data: AuditLogData::EmoteSet {
// 								target_id: self.emote_set.id,
// 								data: AuditLogEmoteSetData::RenameEmote {
// 									emote_id: id.id(),
// 									old_name: emote_set_emote.alias.clone(),
// 									new_name: name.clone(),
// 								},
// 							},
// 							updated_at: Utc::now(),
// 							search_updated_at: None,
// 						})
// 						.session(&mut session)
// 						.await
// 						.map_err(|e| {
// 							tracing::error!(error = %e, "failed to insert audit log");
// 							ApiError::INTERNAL_SERVER_ERROR
// 						})?;

// 					let old_active_emote = ActiveEmoteModel::from_db(
// 						emote_set_emote.clone(),
// 						Some(EmotePartialModel::from_db(
// 							global
// 								.emote_by_id_loader
// 								.load(id.id())
// 								.await
// 								.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
// 								.ok_or(ApiError::NOT_FOUND)?,
// 							None,
// 							&global.config.api.cdn_origin,
// 						)),
// 					);

// 					let mut new_active_emote = old_active_emote.clone();
// 					new_active_emote.name = name.clone();

// 					let old_active_emote =
// serde_json::to_value(old_active_emote).map_err(|e| { 						tracing::error!
// (error = %e, "failed to serialize emote");
// 						ApiError::INTERNAL_SERVER_ERROR
// 					})?;
// 					let new_active_emote =
// serde_json::to_value(new_active_emote).map_err(|e| { 						tracing::error!
// (error = %e, "failed to serialize emote");
// 						ApiError::INTERNAL_SERVER_ERROR
// 					})?;

// 					global
// 						.event_api
// 						.dispatch_event(
// 							EventType::UpdateEmoteSet,
// 							ChangeMap {
// 								id: self.emote_set.id.cast(),
// 								kind: ObjectKind::EmoteSet,
// 								actor: Some(UserPartialModel::from_db(
// 									user.clone(),
// 									None,
// 									None,
// 									&global.config.api.cdn_origin,
// 								)),
// 								updated: vec![ChangeField {
// 									key: "emotes".to_string(),
// 									index: Some(index),
// 									ty: ChangeFieldType::Object,
// 									old_value: old_active_emote,
// 									value: new_active_emote,
// 									..Default::default()
// 								}],
// 								..Default::default()
// 							},
// 							self.emote_set.id,
// 						)
// 						.await
// 						.map_err(|e| {
// 							tracing::error!(error = %e, "failed to dispatch event");
// 							ApiError::INTERNAL_SERVER_ERROR
// 						})?;

// 					emote_set
