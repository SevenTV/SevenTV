use std::hash::{DefaultHasher, Hash, Hasher};
use std::iter;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use itertools::Itertools;
use sha2::Digest;
use shared::database::emote::EmoteFlags;
use shared::database::event::{EventEmoteData, EventEmoteSetData, EventUserData};
use shared::database::Id;
use shared::event::{EventPayload, EventPayloadData};
use shared::event_api::payload::Dispatch;
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, EventType};
use shared::event_api::{self, Message};
use shared::old_types::{EmoteFlagsModel, UserConnectionPartialModel};

use crate::global::Global;
use crate::http::v3::emote_set_loader::load_emote_set;
use crate::http::v3::rest::types::{ActiveEmoteModel, EmotePartialModel, EmoteSetModel, UserConnectionModel};

pub struct EventApi {
	nats: async_nats::Client,
	prefix: String,
	sequence: AtomicU64,
}

#[derive(thiserror::Error, Debug)]
pub enum EventApiError {
	#[error("publish error: {0}")]
	PublishError(#[from] async_nats::PublishError),
	#[error("serialize error: {0}")]
	Serialize(#[from] serde_json::Error),
	#[error("failed to load ressource")]
	LoadRessource,
}

impl EventApi {
	pub fn new(nats: async_nats::Client, prefix: &str) -> Self {
		Self {
			nats,
			sequence: AtomicU64::new(0),
			prefix: prefix.to_string(),
		}
	}

	pub async fn dispatch_old_event<S>(
		&self,
		ty: EventType,
		body: ChangeMap,
		condition_object_id: Id<S>,
	) -> Result<(), EventApiError> {
		let mut nats_subject = vec![];
		nats_subject.push(self.prefix.clone());
		nats_subject.push("events.op.dispatch.type".to_string());
		nats_subject.push(ty.to_string());

		let mut hasher = sha2::Sha256::new();

		hasher.update("object_id");
		hasher.update(condition_object_id.to_string());

		let cond_hash = hex::encode(hasher.finalize());

		nats_subject.push(cond_hash);

		let nats_subject = nats_subject.join(".");

		let mut hasher = DefaultHasher::new();
		hasher.write(&body.id.into_bytes());
		hasher.write(body.kind.as_str().as_bytes());
		body.object.hash(&mut hasher);
		let hash = hasher.finish();

		let message = Message::new(
			Dispatch {
				ty,
				body,
				hash: Some(hash as u32),
				effect: None,
				matches: vec![],
				condition: vec![iter::once(("object_id".to_string(), condition_object_id.to_string())).collect()],
				whisper: None,
			},
			self.sequence.fetch_add(1, Ordering::SeqCst),
		);

		tracing::debug!(subject = %nats_subject, message = ?message, "dispatching event");

		self.nats
			.publish(nats_subject, serde_json::to_string(&message)?.into_bytes().into())
			.await?;

		Ok(())
	}

	pub async fn dispatch_event(
		&self,
		global: &Arc<Global>,
		events: impl IntoIterator<Item = EventPayload>,
	) -> Result<(), EventApiError> {
		let events = events.into_iter();

		let events = events
			.filter_map(|e| Some((e.data.event_api_event_type()?, e.data.id()?, e.data.event_api_kind(), e)))
			.into_group_map_by(|(t, id, k, _)| (*t, *id, *k));

		for ((event_type, id, kind), events) in events {
			let mut updated = vec![];
			let mut pushed = vec![];
			let mut pulled = vec![];
			let mut versions_nested = vec![];

			for (_, _, _, payload) in events {
				match payload.data {
					EventPayloadData::Emote {
						data: EventEmoteData::ChangeName { old, new },
						..
					} => {
						let change = ChangeField {
							key: "name".to_string(),
							ty: ChangeFieldType::String,
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						};
						updated.push(change.clone());
						versions_nested.push(change);
					}
					EventPayloadData::Emote {
						data: EventEmoteData::ChangeFlags { old, new },
						..
					} => {
						if old.contains(EmoteFlags::PublicListed) != new.contains(EmoteFlags::PublicListed) {
							let change = ChangeField {
								key: "listed".to_string(),
								ty: ChangeFieldType::Bool,
								old_value: old.contains(EmoteFlags::PublicListed).into(),
								value: new.contains(EmoteFlags::PublicListed).into(),
								..Default::default()
							};
							updated.push(change.clone());
							versions_nested.push(change);

							// convert to old model
							let old = EmoteFlagsModel::from(old);
							let new = EmoteFlagsModel::from(new);

							if old != new {
								updated.push(ChangeField {
									key: "flags".to_string(),
									ty: ChangeFieldType::Number,
									old_value: old.bits().into(),
									value: new.bits().into(),
									..Default::default()
								});
							}
						}
					}
					EventPayloadData::Emote {
						data: EventEmoteData::ChangeOwner { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "owner_id".to_string(),
							ty: ChangeFieldType::String,
							old_value: old.to_string().into(),
							value: new.to_string().into(),
							..Default::default()
						});
					}
					EventPayloadData::Emote {
						data: EventEmoteData::ChangeTags { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "tags".to_string(),
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						});
					}
					EventPayloadData::EmoteSet {
						data: EventEmoteSetData::AddEmote { emote_id, .. },
						after,
					} => {
						let (index, active_emote) = after.emotes.into_iter().find_position(|e| e.id == emote_id).unwrap();

						// TODO: kind of suboptimal to load the emote here again
						let emote = global
							.emote_by_id_loader
							.load(emote_id)
							.await
							.ok()
							.flatten()
							.ok_or(EventApiError::LoadRessource)?;

						let active_emote = ActiveEmoteModel::from_db(
							active_emote,
							Some(EmotePartialModel::from_db(emote, None, &global.config.api.cdn_origin)),
						);
						let active_emote = serde_json::to_value(active_emote)?;

						pushed.push(ChangeField {
							key: "emotes".to_string(),
							index: Some(index),
							ty: ChangeFieldType::Object,
							value: active_emote,
							..Default::default()
						});
					}
					EventPayloadData::EmoteSet {
						data: EventEmoteSetData::RemoveEmote { emote_id, .. },
						after,
					} => {
						let (index, active_emote) = after.emotes.into_iter().find_position(|e| e.id == emote_id).unwrap();

						// TODO: kind of suboptimal to load the emote here again
						let emote = global
							.emote_by_id_loader
							.load(emote_id)
							.await
							.ok()
							.flatten()
							.ok_or(EventApiError::LoadRessource)?;

						let active_emote = ActiveEmoteModel::from_db(
							active_emote,
							Some(EmotePartialModel::from_db(emote, None, &global.config.api.cdn_origin)),
						);
						let active_emote = serde_json::to_value(active_emote)?;

						pulled.push(ChangeField {
							key: "emotes".to_string(),
							index: Some(index),
							ty: ChangeFieldType::Object,
							old_value: active_emote,
							..Default::default()
						});
					}
					EventPayloadData::EmoteSet {
						data: EventEmoteSetData::RenameEmote {
							emote_id,
							new_alias: new_name,
							..
						},
						after,
					} => {
						let (index, active_emote) = after.emotes.into_iter().find_position(|e| e.id == emote_id).unwrap();

						let old_active_emote = ActiveEmoteModel::from_db(
							active_emote.clone(),
							Some(EmotePartialModel::from_db(
								global
									.emote_by_id_loader
									.load(emote_id)
									.await
									.ok()
									.flatten()
									.ok_or(EventApiError::LoadRessource)?,
								None,
								&global.config.api.cdn_origin,
							)),
						);

						let mut new_active_emote = old_active_emote.clone();
						new_active_emote.name = new_name;

						let old_active_emote = serde_json::to_value(old_active_emote)?;
						let new_active_emote = serde_json::to_value(new_active_emote)?;

						updated.push(ChangeField {
							key: "emotes".to_string(),
							index: Some(index),
							ty: ChangeFieldType::Object,
							old_value: old_active_emote,
							value: new_active_emote,
							..Default::default()
						});
					}
					EventPayloadData::EmoteSet {
						data: EventEmoteSetData::ChangeName { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "name".to_string(),
							ty: ChangeFieldType::String,
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						});
					}
					EventPayloadData::EmoteSet {
						data: EventEmoteSetData::ChangeCapacity { old, new },
						..
					} => {
						updated.push(ChangeField {
							key: "capacity".to_string(),
							ty: ChangeFieldType::Number,
							old_value: old.into(),
							value: new.into(),
							..Default::default()
						});
					}
					EventPayloadData::EmoteSet {
						data: EventEmoteSetData::Delete,
						..
					} => {
						// no updates
						// do not remove this match arm, this is a valid event
					}
					EventPayloadData::User {
						after,
						data: EventUserData::AddConnection { platform },
					} => {
						let user = global
							.user_loader
							.load_fast_user(global, after)
							.await
							.map_err(|_| EventApiError::LoadRessource)?;

						let (index, new_connection) = user
							.user
							.connections
							.into_iter()
							.find_position(|c| c.platform == platform)
							.ok_or(EventApiError::LoadRessource)?;

						let value = serde_json::to_value(UserConnectionModel::from(UserConnectionPartialModel::from_db(
							new_connection,
							user.user.style.active_emote_set_id,
							user.computed.permissions.emote_set_capacity.unwrap_or_default(),
						)))?;

						pushed.push(ChangeField {
							key: "connections".to_string(),
							ty: ChangeFieldType::Object,
							index: Some(index),
							value,
							..Default::default()
						});
					}
					EventPayloadData::User {
						after,
						data: EventUserData::RemoveConnection { connection },
					} => {
						let user = global
							.user_loader
							.load_fast_user(global, after)
							.await
							.map_err(|_| EventApiError::LoadRessource)?;

						let value = serde_json::to_value(UserConnectionModel::from(UserConnectionPartialModel::from_db(
							connection,
							user.user.style.active_emote_set_id,
							user.computed.permissions.emote_set_capacity.unwrap_or_default(),
						)))?;

						pulled.push(ChangeField {
							key: "connections".to_string(),
							ty: ChangeFieldType::Object,
							index: Some(user.connections.len()),
							value,
							..Default::default()
						});
					}
					EventPayloadData::User {
						after,
						data: EventUserData::ChangeActiveEmoteSet { old, new },
					} => {
						// we have to emit the event for every connection since you could have different sets for every connection before

						let emote_set = if let Some(emote_set_id) = new {
							// check if set exists
							global
								.emote_set_by_id_loader
								.load(emote_set_id)
								.await
								.ok()
								.flatten()
								.ok_or(EventApiError::LoadRessource)?
						} else {
							continue;
						};

						let old_set = match old {
							Some(id) => {
								let set = global
									.emote_set_by_id_loader
									.load(id)
									.await
									.map_err(|_| EventApiError::LoadRessource)?;

								if let Some(set) = set {
									let emotes = load_emote_set(global, set.emotes.clone(), None, false)
										.await
										.map_err(|_| EventApiError::LoadRessource)?;

									Some(EmoteSetModel::from_db(set, emotes, None))
								} else {
									None
								}
							}
							None => None,
						};
						let old_set = serde_json::to_value(old_set)?;

						let new_set_id = emote_set.id;
						let emotes = load_emote_set(global, emote_set.emotes.clone(), None, false)
							.await
							.map_err(|_| EventApiError::LoadRessource)?;
						let new_set = EmoteSetModel::from_db(emote_set, emotes, None);
						let new_set = serde_json::to_value(new_set)?;

						for i in 0..after.connections.len() {
							let value = vec![
								ChangeField {
									key: "emote_set".to_string(),
									ty: ChangeFieldType::Object,
									old_value: old_set.clone(),
									value: new_set.clone(),
									..Default::default()
								},
								ChangeField {
									key: "emote_set_id".to_string(),
									ty: ChangeFieldType::String,
									old_value: after.style.active_emote_set_id.map(|id| id.to_string()).into(),
									value: new_set_id.to_string().into(),
									..Default::default()
								},
							];

							let value = serde_json::to_value(value)?;

							updated.push(ChangeField {
								key: "connections".to_string(),
								index: Some(i),
								nested: true,
								value,
								..Default::default()
							});
						}
					}
					EventPayloadData::User {
						data: EventUserData::ChangeActivePaint { old, new },
						..
					} => {
						let mut changes = vec![];

						let old_paint = match old {
							Some(paint_id) => global
								.paint_by_id_loader
								.load(paint_id)
								.await
								.map_err(|_| EventApiError::LoadRessource)?,
							None => None,
						};

						let paint = match new {
							Some(paint_id) => global
								.paint_by_id_loader
								.load(paint_id)
								.await
								.map_err(|_| EventApiError::LoadRessource)?,
							None => None,
						};

						changes.push(ChangeField {
							key: "paint_id".to_string(),
							ty: ChangeFieldType::String,
							value: new.map(|id| id.to_string()).into(),
							old_value: old.map(|id| id.to_string()).into(),
							..Default::default()
						});
						changes.push(ChangeField {
							key: "paint".to_string(),
							ty: ChangeFieldType::Object,
							value: serde_json::to_value(paint)?,
							old_value: serde_json::to_value(old_paint)?,
							..Default::default()
						});

						updated.push(ChangeField {
							key: "style".to_string(),
							ty: ChangeFieldType::Object,
							nested: true,
							value: serde_json::to_value(changes)?,
							..Default::default()
						});
					}
					EventPayloadData::User {
						data: EventUserData::ChangeActiveBadge { old, new },
						..
					} => {
						let mut changes = vec![];

						let old_badge = match old {
							Some(badge_id) => global
								.badge_by_id_loader
								.load(badge_id)
								.await
								.map_err(|_| EventApiError::LoadRessource)?,
							None => None,
						};

						let badge = match new {
							Some(badge_id) => global
								.badge_by_id_loader
								.load(badge_id)
								.await
								.map_err(|_| EventApiError::LoadRessource)?,
							None => None,
						};

						changes.push(ChangeField {
							key: "badge_id".to_string(),
							ty: ChangeFieldType::String,
							value: new.map(|id| id.to_string()).into(),
							old_value: old.map(|id| id.to_string()).into(),
							..Default::default()
						});
						changes.push(ChangeField {
							key: "badge".to_string(),
							ty: ChangeFieldType::Object,
							value: serde_json::to_value(badge)?,
							old_value: serde_json::to_value(old_badge)?,
							..Default::default()
						});

						updated.push(ChangeField {
							key: "style".to_string(),
							ty: ChangeFieldType::Object,
							nested: true,
							value: serde_json::to_value(changes)?,
							..Default::default()
						});
					}
					_ => continue,
				}
			}

			if !versions_nested.is_empty() {
				let versions_nested = serde_json::to_value(versions_nested)?;

				updated.push(ChangeField {
					key: "versions".to_string(),
					nested: true,
					index: Some(0),
					value: versions_nested,
					..Default::default()
				});
			}

			// TODO: actor
			let body = event_api::types::ChangeMap {
				id,
				kind,
				updated,
				pushed,
				pulled,
				..Default::default()
			};

			self.dispatch_old_event(event_type, body, id).await?;
		}

		Ok(())
	}
}
