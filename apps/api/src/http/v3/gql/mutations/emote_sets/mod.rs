use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use chrono::Utc;
use emote_add::emote_add;
use emote_remove::emote_remove;
use emote_update::emote_update;
use hyper::StatusCode;
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::emote::EmoteId;
use shared::database::emote_set::{EmoteSet as DbEmoteSet, EmoteSetKind};
use shared::database::event::EventEmoteSetData;
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{EmoteSetPermission, PermissionsExt, UserPermission};
use shared::database::user::editor::{
	EditorEmoteSetPermission, EditorPermission, EditorUserPermission, UserEditorId, UserEditorState,
};
use shared::database::user::FullUserRef;
use shared::event::{EventPayload, EventPayloadData};
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, EventType, ObjectKind};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::UserPartialModel;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::emote_set::{ActiveEmote, EmoteSet};
use crate::http::v3::gql::types::ListItemAction;
use crate::transactions::{with_transaction, TransactionError};

mod emote_add;
mod emote_remove;
mod emote_update;

#[derive(Default)]
pub struct EmoteSetsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetsMutation {
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<EmoteSetOps>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

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
					.user_loader
					.load(global, user_id.id())
					.await
					.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
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
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: user.id,
					editor_id: user_id.id(),
				})
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
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

		let res = with_transaction(global, |mut tx| async move {
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
				updated_at: Utc::now(),
				search_updated_at: None,
				emotes_changed_since_reindex: false,
			};

			tx.insert_one::<DbEmoteSet>(&emote_set, None).await?;

			tx.register_event(EventPayload {
				actor_id: Some(user.id),
				data: EventPayloadData::EmoteSet {
					after: emote_set.clone(),
					data: EventEmoteSetData::Create,
				},
				timestamp: Utc::now(),
			})?;

			Ok(emote_set)
		})
		.await;

		match res {
			Ok(emote_set) => Ok(EmoteSet::from_db(emote_set)),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
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
		editor_perm: impl Into<EditorPermission>,
	) -> Result<FullUserRef<'a>, ApiError> {
		let mut editor_perm = editor_perm.into();
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

				return Ok(target);
			}
			EmoteSetKind::Special => {
				if !user.has(EmoteSetPermission::ManageSpecial) {
					return Err(ApiError::new_const(
						StatusCode::FORBIDDEN,
						"this user does not have permission to manage special emote sets",
					));
				}

				return Ok(target);
			}
			EmoteSetKind::Personal | EmoteSetKind::Normal => {}
		}

		let owner_id = self
			.emote_set
			.owner_id
			.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "owner not found"))?;

		// If the person who is updating the set is not the owner, we need to load the
		// owner.
		if owner_id != user.id {
			target = FullUserRef::Owned(
				global
					.user_loader
					.load(global, owner_id)
					.await
					.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "owner not found"))?,
			)
		}

		let mut forbidden_msg = "you do not have permission to manage this user's emote sets";

		// If the emote set is personal, check if the owner has permission to use
		if matches!(self.emote_set.kind, EmoteSetKind::Personal) {
			if !target.has(UserPermission::UsePersonalEmoteSet) {
				return Err(ApiError::new_const(
					StatusCode::FORBIDDEN,
					"this user does not have permission to use personal emote sets",
				));
			}

			editor_perm = EditorUserPermission::ManagePersonalEmoteSet.into();
			forbidden_msg = "you do not have permission to manage this user's personal emote set";
		}

		if !target.has(EmoteSetPermission::Manage) && !user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::new_const(
				StatusCode::FORBIDDEN,
				"the target user does not have permission to use emote sets",
			));
		}

		if target.id != user.id && !user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: owner_id,
					editor_id: user.id,
				})
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::new_const(
					StatusCode::NOT_FOUND,
					"you are not an editor for this user",
				))?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(editor_perm) {
				return Err(ApiError::new_const(StatusCode::FORBIDDEN, forbidden_msg));
			}
		}

		Ok(target)
	}
}

#[derive(InputObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct UpdateEmoteSetInput {
	name: Option<String>,
	capacity: Option<u32>,
	origins: Option<Vec<EmoteSetOriginInput>>,
}

#[derive(InputObject, Clone)]
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
		let actor = auth_session.user(global).await?;

		let target = self
			.check_perms(global, auth_session, EditorEmoteSetPermission::Manage)
			.await?;
		let target = target.as_ref();

		let id: EmoteId = id.id();

		let res = with_transaction(&global, |tx| async move {
			match action {
				ListItemAction::Add => emote_add(global, tx, actor, target, &self.emote_set, id, name).await,
				ListItemAction::Remove => emote_remove(global, tx, actor, &self.emote_set, id).await,
				ListItemAction::Update => emote_update(global, tx, actor, &self.emote_set, id, name).await,
			}
		})
		.await;

		match res {
			Ok(emote_set) => Ok(emote_set.emotes.into_iter().map(ActiveEmote::from_db).collect()),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
	}

	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Manage)")]
	async fn update<'ctx>(&self, ctx: &Context<'ctx>, data: UpdateEmoteSetInput) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let target = self
			.check_perms(global, auth_session, EditorEmoteSetPermission::Manage)
			.await?;

		let res = with_transaction(global, |mut tx| async move {
			let mut changes = vec![];

			let new_name = if let Some(name) = data.name {
				// TODO validate this name

				changes.push(ChangeField {
					key: "name".to_string(),
					ty: ChangeFieldType::String,
					old_value: self.emote_set.name.clone().into(),
					value: name.clone().into(),
					..Default::default()
				});

				Some(name)
			} else {
				None
			};

			let new_capacity = if let Some(capacity) = data.capacity {
				if capacity > i32::MAX as u32 {
					return Err(TransactionError::custom(ApiError::new_const(
						StatusCode::BAD_REQUEST,
						"emote set capacity is too large",
					)));
				}

				if capacity == 0 {
					return Err(TransactionError::custom(ApiError::new_const(
						StatusCode::BAD_REQUEST,
						"emote set capacity cannot be 0",
					)));
				}

				if capacity < self.emote_set.emotes.len() as u32 {
					return Err(TransactionError::custom(ApiError::new_const(
						StatusCode::BAD_REQUEST,
						"emote set capacity cannot be less than the number of emotes in the set",
					)));
				}

				if capacity as i32 > target.computed.permissions.emote_set_capacity.unwrap_or_default().max(0) {
					return Err(TransactionError::custom(ApiError::new_const(
						StatusCode::BAD_REQUEST,
						"emote set capacity cannot exceed user's capacity",
					)));
				}

				changes.push(ChangeField {
					key: "capacity".to_string(),
					ty: ChangeFieldType::Number,
					old_value: self.emote_set.capacity.into(),
					value: capacity.into(),
					..Default::default()
				});

				Some(capacity as i32)
			} else {
				None
			};

			if data.origins.is_some() {
				return Err(TransactionError::custom(ApiError::new_const(
					StatusCode::BAD_REQUEST,
					"legacy origins are not supported",
				)));
			}

			let emote_set = tx
				.find_one_and_update(
					filter::filter! {
						DbEmoteSet {
							#[query(rename = "_id")]
							id: self.emote_set.id,
						}
					},
					update::update! {
						#[query(set)]
						DbEmoteSet {
							#[query(optional)]
							name: new_name.as_ref(),
							#[query(optional)]
							capacity: new_capacity,
							updated_at: chrono::Utc::now(),
						}
					},
					FindOneAndUpdateOptions::builder()
						.return_document(ReturnDocument::After)
						.build(),
				)
				.await?
				.ok_or(ApiError::INTERNAL_SERVER_ERROR)
				.map_err(TransactionError::custom)?;

			if let Some(new_name) = new_name {
				tx.register_event(EventPayload {
					actor_id: Some(auth_session.user_id()),
					data: EventPayloadData::EmoteSet {
						after: emote_set.clone(),
						data: EventEmoteSetData::ChangeName {
							old: self.emote_set.name.clone(),
							new: new_name,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if let Some(new_capacity) = new_capacity {
				tx.register_event(EventPayload {
					actor_id: Some(auth_session.user_id()),
					data: EventPayloadData::EmoteSet {
						after: emote_set.clone(),
						data: EventEmoteSetData::ChangeCapacity {
							old: self.emote_set.capacity,
							new: Some(new_capacity as i32),
						},
					},
					timestamp: Utc::now(),
				})?;
			}

			if !changes.is_empty() {
				global
					.event_api
					.dispatch_event(
						EventType::UpdateEmoteSet,
						ChangeMap {
							id: self.id.0,
							kind: ObjectKind::EmoteSet,
							actor: Some(UserPartialModel::from_db(
								auth_session.user(global).await.map_err(TransactionError::custom)?.clone(),
								None,
								None,
								&global.config.api.cdn_origin,
							)),
							updated: changes,
							..Default::default()
						},
						self.emote_set.id,
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to dispatch event");
						ApiError::INTERNAL_SERVER_ERROR
					})
					.map_err(TransactionError::custom)?;
			}

			Ok(emote_set)
		})
		.await;

		match res {
			Ok(emote_set) => Ok(EmoteSet::from_db(emote_set)),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
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

		let res = with_transaction(global, |mut tx| async move {
			let emote_set = tx
				.find_one_and_delete(
					filter::filter!(DbEmoteSet {
						#[query(rename = "_id")]
						id: self.emote_set.id
					}),
					None,
				)
				.await?;

			if let Some(emote_set) = emote_set {
				tx.register_event(EventPayload {
					actor_id: Some(auth_session.user_id()),
					data: EventPayloadData::EmoteSet {
						after: emote_set,
						data: EventEmoteSetData::Delete,
					},
					timestamp: Utc::now(),
				})?;

				let body = ChangeMap {
					id: self.emote_set.id.cast(),
					kind: ObjectKind::EmoteSet,
					actor: Some(UserPartialModel::from_db(
						auth_session.user(global).await.map_err(TransactionError::custom)?.clone(),
						None,
						None,
						&global.config.api.cdn_origin,
					)),
					..Default::default()
				};

				global
					.event_api
					.dispatch_event(EventType::DeleteEmoteSet, body, self.emote_set.id)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to dispatch event");
						ApiError::INTERNAL_SERVER_ERROR
					})
					.map_err(TransactionError::custom)?;

				Ok(true)
			} else {
				Ok(false)
			}
		})
		.await;

		match res {
			Ok(deleted) => Ok(deleted),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
	}
}
