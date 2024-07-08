use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use hyper::StatusCode;
use itertools::Itertools;
use mongodb::bson::{doc, to_bson};
use mongodb::options::ReturnDocument;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteSetData, AuditLogId};
use shared::database::emote::EmoteFlags;
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestId, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::emote_set::{EmoteSet as DbEmoteSet, EmoteSetEmote, EmoteSetKind};
use shared::database::role::permissions::{EmoteSetPermission, PermissionsExt, UserPermission};
use shared::database::user::editor::{EditorEmoteSetPermission, EditorUserPermission, UserEditorState};
use shared::database::user::FullUserRef;
use shared::database::Collection;
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, EventType, ObjectKind};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::UserPartialModel;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::emote_set::{ActiveEmote, EmoteSet};
use crate::http::v3::gql::types::ListItemAction;
use crate::http::v3::rest::types::{ActiveEmoteModel, EmotePartialModel};

#[derive(Default)]
pub struct EmoteSetsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetsMutation {
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<EmoteSetOps>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_set = global
			.emote_set_by_id_loader()
			.load(id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote_set.map(|s| EmoteSetOps {
			id: s.id.into(),
			emote_set: s,
		}))
	}

	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Manage)")]
	async fn create_emote_set<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		user_id: GqlObjectId,
		data: CreateEmoteSetInput,
	) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await?;
		let other_user = if user_id.id() == user.id {
			None
		} else {
			Some(
				global
					.user_loader()
					.load(global, user_id.id())
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "user not found"))?,
			)
		};

		let target = other_user.as_ref().unwrap_or(&user);

		if !target.has(EmoteSetPermission::Manage) && !user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::new_const(
				StatusCode::FORBIDDEN,
				"this user does not have permission to create emote sets",
			));
		}

		if target.id != user.id && !user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader()
				.load((user_id.id(), user.id))
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::new_const(
					StatusCode::NOT_FOUND,
					"you are not an editor for this user",
				))?;

			if editor.state != UserEditorState::Accepted
				|| !editor.permissions.has_emote_set(EditorEmoteSetPermission::Create)
			{
				return Err(ApiError::new_const(
					StatusCode::FORBIDDEN,
					"you do not have permission to create emote sets for this user",
				));
			}
		}

		if data.privileged.unwrap_or(false) {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"privileged emote sets are not supported",
			));
		}

		let capacity = target.computed.permissions.emote_set_capacity.unwrap_or_default().max(0);

		if capacity == 0 {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"maximum emote set capacity is 0, cannot create emote set",
			));
		}

		let emote_set = DbEmoteSet {
			id: Default::default(),
			owner_id: Some(user_id.id()),
			name: data.name,
			capacity: Some(capacity),
			description: None,
			emotes: vec![],
			kind: EmoteSetKind::Normal,
			origin_config: None,
			tags: vec![],
		};

		let mut session = global.mongo().start_session().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		DbEmoteSet::collection(global.db())
			.insert_one(&emote_set)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert emote set");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		AuditLog::collection(global.db())
			.insert_one(AuditLog {
				id: AuditLogId::new(),
				actor_id: Some(user.id),
				data: AuditLogData::EmoteSet {
					target_id: emote_set.id,
					data: AuditLogEmoteSetData::Create,
				},
			})
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert audit log");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		session.commit_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(EmoteSet::from_db(emote_set))
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateEmoteSetInput {
	name: String,
	privileged: Option<bool>,
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteSetOps {
	id: GqlObjectId,
	#[graphql(skip)]
	emote_set: DbEmoteSet,
}

impl EmoteSetOps {
	async fn check_perms<'a>(
		&self,
		global: &Arc<Global>,
		auth_session: &'a AuthSession,
		editor_perm: EditorEmoteSetPermission,
	) -> Result<FullUserRef<'a>, ApiError> {
		let user = auth_session.user(global).await?;

		let mut target = FullUserRef::Ref(&user);

		match self.emote_set.kind {
			EmoteSetKind::Global => {
				if !user.has(EmoteSetPermission::ManageGlobal) {
					return Err(ApiError::new_const(
						StatusCode::FORBIDDEN,
						"this user does not have permission to manage global emote sets",
					));
				}
			}
			EmoteSetKind::Special => {
				if !user.has(EmoteSetPermission::ManageSpecial) {
					return Err(ApiError::new_const(
						StatusCode::FORBIDDEN,
						"this user does not have permission to manage special emote sets",
					));
				}
			}
			EmoteSetKind::Personal | EmoteSetKind::Normal => {
				let owner_id = self
					.emote_set
					.owner_id
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "owner not found"))?;

				if owner_id != user.id {
					target = FullUserRef::Owned(
						global
							.user_loader()
							.load(global, owner_id)
							.await
							.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
							.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "owner not found"))?,
					)
				}

				if matches!(self.emote_set.kind, EmoteSetKind::Personal) {
					if !target.has(UserPermission::UsePersonalEmoteSet) {
						return Err(ApiError::new_const(
							StatusCode::FORBIDDEN,
							"this user does not have permission to use personal emote sets",
						));
					}

					if target.id != user.id
						&& !user.has_all([UserPermission::ManageAny.into(), EmoteSetPermission::ManageAny.into()])
					{
						let editor = global
							.user_editor_by_id_loader()
							.load((owner_id, user.id))
							.await
							.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
							.ok_or(ApiError::new_const(
								StatusCode::NOT_FOUND,
								"you are not an editor for this user",
							))?;

						if editor.state != UserEditorState::Accepted
							|| !editor.permissions.has_user(EditorUserPermission::ManagePersonalEmoteSet)
						{
							return Err(ApiError::new_const(
								StatusCode::FORBIDDEN,
								"you do not have permission to manage this user's personal emote set",
							));
						}
					}
				} else {
					if !target.has(EmoteSetPermission::Manage) && !user.has(EmoteSetPermission::ManageAny) {
						return Err(ApiError::new_const(
							StatusCode::FORBIDDEN,
							"this user does not have permission to manage emote sets",
						));
					}

					if target.id != user.id && !user.has(EmoteSetPermission::ManageAny) {
						let editor = global
							.user_editor_by_id_loader()
							.load((owner_id, user.id))
							.await
							.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
							.ok_or(ApiError::new_const(
								StatusCode::NOT_FOUND,
								"you are not an editor for this user",
							))?;

						if editor.state != UserEditorState::Accepted || !editor.permissions.has_emote_set(editor_perm) {
							return Err(ApiError::new_const(
								StatusCode::FORBIDDEN,
								"you do not have permission to manage this user's emote sets",
							));
						}
					}
				}
			}
		}

		Ok(target)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UpdateEmoteSetInput {
	name: Option<String>,
	capacity: Option<u32>,
	origins: Option<Vec<EmoteSetOriginInput>>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSetOriginInput {
	id: GqlObjectId,
	weight: Option<u32>,
	slices: Option<Vec<u32>>,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetOps {
	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Manage)")]
	async fn emotes<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: GqlObjectId,
		action: ListItemAction,
		name: Option<String>,
	) -> Result<Vec<ActiveEmote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;
		let user = auth_session.user(global).await?;
		self.check_perms(global, auth_session, EditorEmoteSetPermission::Manage)
			.await?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let mut session = global.mongo().start_session().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let emote_set = match action {
			ListItemAction::Add => {
				if let Some(capacity) = self.emote_set.capacity {
					if self.emote_set.emotes.len() as i32 >= capacity {
						return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "emote set is at capacity"));
					}
				}

				let emote = global
					.emote_by_id_loader()
					.load(id.id())
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::NOT_FOUND)?;

				let alias = name.unwrap_or_else(|| emote.default_name.clone());

				if self.emote_set.emotes.iter().any(|e| e.alias == alias || e.id == id.id()) {
					return Err(ApiError::new_const(
						StatusCode::CONFLICT,
						"this emote is already in the set or has a conflicting name",
					));
				}

				if matches!(self.emote_set.kind, EmoteSetKind::Personal) {
					if emote.flags.contains(EmoteFlags::DeniedPersonal) {
						return Err(ApiError::new_const(
							StatusCode::BAD_REQUEST,
							"emote is not allowed in personal emote sets",
						));
					} else if !emote.flags.contains(EmoteFlags::ApprovedPersonal) {
						let inserted_id = EmoteModerationRequestId::new();

						let result = EmoteModerationRequest::collection(global.db())
							.find_one_and_update(
								doc! {
									"kind": to_bson(&EmoteModerationRequestKind::PersonalUse).unwrap(),
									"emote_id": emote.id,
								},
								doc! {
									"$setOnInsert": to_bson(&EmoteModerationRequest {
										id: inserted_id,
										emote_id: emote.id,
										country_code: None,
										kind: EmoteModerationRequestKind::PersonalUse,
										assigned_to: vec![],
										priority: user.computed.permissions.emote_moderation_request_priority.unwrap_or_default(),
										reason: Some("User requested to add emote to a personal set".to_string()),
										status: EmoteModerationRequestStatus::Pending,
										user_id: user.id,
									}).unwrap(),
								},
							)
							.upsert(true)
							.session(&mut session)
							.await
							.map_err(|e| {
								tracing::error!(error = %e, "failed to insert moderation request");
								ApiError::INTERNAL_SERVER_ERROR
							})?
							.ok_or_else(|| {
								tracing::error!("failed to insert moderation request");
								ApiError::INTERNAL_SERVER_ERROR
							})?;

						// We only care to check if this is the result we just inserted
						if result.id == inserted_id {
							let count = EmoteModerationRequest::collection(global.db())
								.count_documents(doc! {
									"kind": to_bson(&EmoteModerationRequestKind::PersonalUse).unwrap(),
									"user_id": user.id,
									"status": to_bson(&EmoteModerationRequestStatus::Pending).unwrap(),
								})
								.session(&mut session)
								.await
								.map_err(|e| {
									tracing::error!(error = %e, "failed to count moderation requests");
									ApiError::INTERNAL_SERVER_ERROR
								})?;

							if count as i32 > user.computed.permissions.emote_moderation_request_limit.unwrap_or_default() {
								return Err(ApiError::new_const(
									StatusCode::BAD_REQUEST,
									"too many pending moderation requests",
								));
							}
						}
					}
				}

				let emote_set_emote = EmoteSetEmote {
					id: id.id(),
					added_by_id: Some(auth_session.user_id()),
					alias: alias.clone(),
					..Default::default()
				};

				let emote_set = DbEmoteSet::collection(global.db())
					.find_one_and_update(
						doc! {
							"_id": self.emote_set.id,
						},
						doc! {
							"$push": {
								"emotes": to_bson(&emote_set_emote).unwrap(),
							},
						},
					)
					.return_document(ReturnDocument::After)
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to add emote to set");
						ApiError::INTERNAL_SERVER_ERROR
					})?
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote set not found"))?;

				if let Some(capacity) = emote_set.capacity {
					if emote_set.emotes.len() as i32 > capacity {
						return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "emote set is at capacity"));
					}
				}

				AuditLog::collection(global.db())
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(auth_session.user_id()),
						data: AuditLogData::EmoteSet {
							target_id: self.emote_set.id,
							data: AuditLogEmoteSetData::AddEmote {
								emote_id: id.id(),
								alias,
							},
						},
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				let active_emote = ActiveEmoteModel::from_db(
					emote_set_emote,
					Some(EmotePartialModel::from_db(emote, None, &global.config().api.cdn_origin)),
				);
				let active_emote = serde_json::to_value(active_emote).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize emote");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

				global
					.event_api()
					.dispatch_event(
						EventType::UpdateEmoteSet,
						ChangeMap {
							id: self.emote_set.id.cast(),
							kind: ObjectKind::EmoteSet,
							actor: Some(UserPartialModel::from_db(
								user.clone(),
								&global_config,
								None,
								None,
								&global.config().api.cdn_origin,
							)),
							pushed: vec![ChangeField {
								key: "emotes".to_string(),
								index: Some(emote_set.emotes.len()),
								ty: ChangeFieldType::Object,
								value: active_emote,
								..Default::default()
							}],
							..Default::default()
						},
						Some(("object_id", self.emote_set.id.to_string())),
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to dispatch event");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				emote_set
			}
			ListItemAction::Remove => {
				let (index, active_emote) = self
					.emote_set
					.emotes
					.iter()
					.find_position(|e| e.id == id.id())
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found in set"))?;

				let emote_set = DbEmoteSet::collection(global.db())
					.find_one_and_update(
						doc! {
							"_id": self.emote_set.id,
						},
						doc! {
							"$pull": {
								"emotes": {
									"id": id.0,
								},
							},
						},
					)
					.session(&mut session)
					.return_document(ReturnDocument::After)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to remove emote from set");
						ApiError::INTERNAL_SERVER_ERROR
					})?
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote set not found"))?;

				AuditLog::collection(global.db())
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(auth_session.user_id()),
						data: AuditLogData::EmoteSet {
							target_id: self.emote_set.id,
							data: AuditLogEmoteSetData::RemoveEmote { emote_id: id.id() },
						},
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				let active_emote = ActiveEmoteModel::from_db(
					active_emote.clone(),
					Some(EmotePartialModel::from_db(
						global
							.emote_by_id_loader()
							.load(id.id())
							.await
							.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
							.ok_or(ApiError::NOT_FOUND)?,
						None,
						&global.config().api.cdn_origin,
					)),
				);
				let active_emote = serde_json::to_value(active_emote).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize emote");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

				global
					.event_api()
					.dispatch_event(
						EventType::UpdateEmoteSet,
						ChangeMap {
							id: self.emote_set.id.cast(),
							kind: ObjectKind::EmoteSet,
							actor: Some(UserPartialModel::from_db(
								user.clone(),
								&global_config,
								None,
								None,
								&global.config().api.cdn_origin,
							)),
							pushed: vec![ChangeField {
								key: "emotes".to_string(),
								index: Some(index),
								ty: ChangeFieldType::Object,
								value: active_emote,
								..Default::default()
							}],
							..Default::default()
						},
						Some(("object_id", self.emote_set.id.to_string())),
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to dispatch event");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				emote_set
			}
			ListItemAction::Update => {
				let (index, emote_set_emote) = self
					.emote_set
					.emotes
					.iter()
					.find_position(|e| e.id == id.id())
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found in set"))?;

				let name = if let Some(name) = name {
					name
				} else {
					let emote = global
						.emote_by_id_loader()
						.load(id.id())
						.await
						.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
						.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote not found"))?;

					emote.default_name
				};

				if emote_set_emote.alias == name {
					return Err(ApiError::new_const(StatusCode::CONFLICT, "emote already has this name"));
				}

				self.emote_set
					.emotes
					.iter()
					.find(|e| e.alias == name)
					.ok_or(ApiError::new_const(
						StatusCode::CONFLICT,
						"emote with this name already exists in set",
					))?;

				let emote_set = DbEmoteSet::collection(global.db())
					.find_one_and_update(
						doc! {
							"_id": self.emote_set.id,
							"emotes.id": id.0,
						},
						doc! {
							"$set": {
								"emotes.$.alias": &name,
							},
						},
					)
					.session(&mut session)
					.return_document(ReturnDocument::After)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to update emote in set");
						ApiError::INTERNAL_SERVER_ERROR
					})?
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "emote set not found"))?;

				AuditLog::collection(global.db())
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(auth_session.user_id()),
						data: AuditLogData::EmoteSet {
							target_id: self.emote_set.id,
							data: AuditLogEmoteSetData::RenameEmote {
								emote_id: id.id(),
								old_name: emote_set_emote.alias.clone(),
								new_name: name.clone(),
							},
						},
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				let old_active_emote = ActiveEmoteModel::from_db(
					emote_set_emote.clone(),
					Some(EmotePartialModel::from_db(
						global
							.emote_by_id_loader()
							.load(id.id())
							.await
							.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
							.ok_or(ApiError::NOT_FOUND)?,
						None,
						&global.config().api.cdn_origin,
					)),
				);

				let mut new_active_emote = old_active_emote.clone();
				new_active_emote.name = name.clone();

				let old_active_emote = serde_json::to_value(old_active_emote).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize emote");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
				let new_active_emote = serde_json::to_value(new_active_emote).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize emote");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

				global
					.event_api()
					.dispatch_event(
						EventType::UpdateEmoteSet,
						ChangeMap {
							id: self.emote_set.id.cast(),
							kind: ObjectKind::EmoteSet,
							actor: Some(UserPartialModel::from_db(
								user.clone(),
								&global_config,
								None,
								None,
								&global.config().api.cdn_origin,
							)),
							updated: vec![
								ChangeField {
									key: "emotes".to_string(),
									index: Some(index),
									ty: ChangeFieldType::Object,
									old_value: old_active_emote,
									value: new_active_emote,
									..Default::default()
								}
							],
							..Default::default()
						},
						Some(("object_id", self.emote_set.id.to_string())),
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to dispatch event");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				emote_set
			}
		};

		session.commit_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(emote_set.emotes.into_iter().map(ActiveEmote::from_db).collect())
	}

	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Manage)")]
	async fn update<'ctx>(&self, ctx: &Context<'ctx>, data: UpdateEmoteSetInput) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let target = self
			.check_perms(global, auth_session, EditorEmoteSetPermission::Manage)
			.await?;

		let mut changes = vec![];

		let mut session = global.mongo().start_session().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let mut update = doc! {};

		if let Some(name) = data.name {
			// TODO validate this name
			update.insert("name", &name);

			AuditLog::collection(global.db())
				.insert_one(AuditLog {
					id: AuditLogId::new(),
					actor_id: Some(auth_session.user_id()),
					data: AuditLogData::EmoteSet {
						target_id: self.emote_set.id,
						data: AuditLogEmoteSetData::ChangeName {
							old: self.emote_set.name.clone(),
							new: name.clone(),
						},
					},
				})
				.session(&mut session)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to insert audit log");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

			changes.push(ChangeField {
				key: "name".to_string(),
				ty: ChangeFieldType::String,
				old_value: self.emote_set.name.clone().into(),
				value: name.into(),
				..Default::default()
			});
		}

		if let Some(capacity) = data.capacity {
			if capacity > i32::MAX as u32 {
				return Err(ApiError::new_const(
					StatusCode::BAD_REQUEST,
					"emote set capacity is too large",
				));
			}

			if capacity == 0 {
				return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "emote set capacity cannot be 0"));
			}

			if capacity < self.emote_set.emotes.len() as u32 {
				return Err(ApiError::new_const(
					StatusCode::BAD_REQUEST,
					"emote set capacity cannot be less than the number of emotes in the set",
				));
			}

			if capacity as i32 > target.computed.permissions.emote_set_capacity.unwrap_or_default().max(0) {
				return Err(ApiError::new_const(
					StatusCode::BAD_REQUEST,
					"emote set capacity cannot exceed user's capacity",
				));
			}

			update.insert("capacity", capacity as i32);

			AuditLog::collection(global.db())
				.insert_one(AuditLog {
					id: AuditLogId::new(),
					actor_id: Some(auth_session.user_id()),
					data: AuditLogData::EmoteSet {
						target_id: self.emote_set.id,
						data: AuditLogEmoteSetData::ChangeCapacity {
							old: self.emote_set.capacity,
							new: Some(capacity as i32),
						},
					},
				})
				.session(&mut session)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to insert audit log");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

			changes.push(ChangeField {
				key: "capacity".to_string(),
				ty: ChangeFieldType::Number,
				old_value: self.emote_set.capacity.into(),
				value: capacity.into(),
				..Default::default()
			});
		}

		if data.origins.is_some() {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"legacy origins are not supported",
			));
		}

		let emote_set = DbEmoteSet::collection(global.db())
			.find_one_and_update(doc! { "_id": self.emote_set.id }, doc! { "$set": update })
			.session(&mut session)
			.return_document(ReturnDocument::After)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update emote set");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		session.commit_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		if !changes.is_empty() {
			let global_config = global
				.global_config_loader()
				.load(())
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

			global
				.event_api()
				.dispatch_event(
					EventType::UpdateEmoteSet,
					ChangeMap {
						id: self.id.0,
						kind: ObjectKind::EmoteSet,
						actor: Some(UserPartialModel::from_db(
							auth_session.user(global).await?.clone(),
							&global_config,
							None,
							None,
							&global.config().api.cdn_origin,
						)),
						updated: changes,
						..Default::default()
					},
					Some(("object_id", self.emote_set.id.to_string())),
				)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to dispatch event");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		}

		Ok(EmoteSet::from_db(emote_set))
	}

	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Manage)")]
	async fn delete<'ctx>(&self, ctx: &Context<'ctx>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		self.check_perms(global, auth_session, EditorEmoteSetPermission::Manage)
			.await?;

		if matches!(
			self.emote_set.kind,
			EmoteSetKind::Personal | EmoteSetKind::Global | EmoteSetKind::Special
		) {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"cannot delete personal, global, or special emote sets",
			));
		}

		let mut session = global.mongo().start_session().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let res = DbEmoteSet::collection(global.db())
			.delete_one(doc! { "_id": self.emote_set.id })
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to delete emote set");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		if res.deleted_count > 0 {
			AuditLog::collection(global.db())
				.insert_one(AuditLog {
					id: AuditLogId::new(),
					actor_id: Some(auth_session.user_id()),
					data: AuditLogData::EmoteSet {
						target_id: self.emote_set.id,
						data: AuditLogEmoteSetData::Delete,
					},
				})
				.session(&mut session)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to insert audit log");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

			let global_config = global
				.global_config_loader()
				.load(())
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

			let body = ChangeMap {
				id: self.emote_set.id.cast(),
				kind: ObjectKind::EmoteSet,
				actor: Some(UserPartialModel::from_db(
					auth_session.user(global).await?.clone(),
					&global_config,
					None,
					None,
					&global.config().api.cdn_origin,
				)),
				..Default::default()
			};

			global
				.event_api()
				.dispatch_event(
					EventType::DeleteEmoteSet,
					body,
					Some(("object_id", self.emote_set.id.to_string())),
				)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to dispatch event");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		}

		session.commit_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(res.deleted_count > 0)
	}
}
