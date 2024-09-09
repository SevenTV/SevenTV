use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use chrono::Utc;
use emote_add::emote_add;
use emote_remove::emote_remove;
use emote_update::emote_update;
use hyper::StatusCode;
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::emote_set::{EmoteSet as DbEmoteSet, EmoteSetKind};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{EmoteSetPermission, PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::editor::{
	EditorEmoteSetPermission, EditorPermission, EditorUserPermission, UserEditorId, UserEditorState,
};
use shared::database::user::FullUserRef;
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::session::Session;
use crate::http::v3::gql::guards::{PermissionGuard, RateLimitGuard};
use crate::http::v3::gql::queries::emote_set::{ActiveEmote, EmoteSet};
use crate::http::v3::gql::types::ListItemAction;
use crate::http::v3::validators::NameValidator;
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

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetCreate, 1))"
	)]
	async fn create_emote_set<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		user_id: GqlObjectId,
		data: CreateEmoteSetInput,
	) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let session = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let authed_user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

		let other_user = if user_id.id() == authed_user.id {
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

		let target = other_user.as_ref().unwrap_or(authed_user);

		if !target.has(EmoteSetPermission::Manage) && !authed_user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::new_const(
				StatusCode::FORBIDDEN,
				"this user does not have permission to create emote sets",
			));
		}

		if target.id != authed_user.id && !authed_user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: authed_user.id,
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

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::EmoteSet {
					after: emote_set.clone(),
					data: InternalEventEmoteSetData::Create,
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
	#[graphql(validator(custom = "NameValidator"))]
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
		session: &'a Session,
		editor_perm: impl Into<EditorPermission>,
	) -> Result<FullUserRef<'a>, ApiError> {
		let mut editor_perm = editor_perm.into();
		let user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

		let mut target = FullUserRef::Ref(user);

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
					.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "owner not found"))?
					.into(),
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
	#[graphql(validator(custom = "NameValidator"))]
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
	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn emotes<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: GqlObjectId,
		action: ListItemAction,
		#[graphql(validator(custom = "NameValidator"))] name: Option<String>,
	) -> Result<Vec<ActiveEmote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let session = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		let res = with_transaction(global, |tx| async move {
			match action {
				ListItemAction::Add => emote_add(global, tx, session, &self.emote_set, id.id(), name).await,
				ListItemAction::Remove => emote_remove(global, tx, session, &self.emote_set, id.id()).await,
				ListItemAction::Update => emote_update(global, tx, session, &self.emote_set, id.id(), name).await,
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

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn update<'ctx>(&self, ctx: &Context<'ctx>, data: UpdateEmoteSetInput) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let sesison = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let target = self.check_perms(global, sesison, EditorEmoteSetPermission::Manage).await?;

		let authed_user = sesison.user().ok_or(ApiError::UNAUTHORIZED)?;

		let res = with_transaction(global, |mut tx| async move {
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
							name: data.name.as_ref(),
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

			if let Some(new_name) = data.name {
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: sesison.user_session_id(),
					data: InternalEventData::EmoteSet {
						after: emote_set.clone(),
						data: InternalEventEmoteSetData::ChangeName {
							old: self.emote_set.name.clone(),
							new: new_name,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if let Some(new_capacity) = new_capacity {
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: sesison.user_session_id(),
					data: InternalEventData::EmoteSet {
						after: emote_set.clone(),
						data: InternalEventEmoteSetData::ChangeCapacity {
							old: self.emote_set.capacity,
							new: Some(new_capacity),
						},
					},
					timestamp: Utc::now(),
				})?;
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

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn delete<'ctx>(&self, ctx: &Context<'ctx>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let session = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		if matches!(
			self.emote_set.kind,
			EmoteSetKind::Personal | EmoteSetKind::Global | EmoteSetKind::Special
		) {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"cannot delete personal, global, or special emote sets",
			));
		}

		let authed_user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

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
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::EmoteSet {
						after: emote_set,
						data: InternalEventEmoteSetData::Delete,
					},
					timestamp: Utc::now(),
				})?;

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
