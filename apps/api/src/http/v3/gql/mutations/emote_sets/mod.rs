use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use chrono::Utc;
use emote_add::emote_add;
use emote_remove::emote_remove;
use emote_update::emote_update;
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

use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::v3::gql::queries::emote_set::{ActiveEmote, EmoteSet};
use crate::http::v3::gql::types::ListItemAction;
use crate::http::validators::{EmoteNameValidator, NameValidator};
use crate::transactions::{transaction, transaction_with_mutex, GeneralMutexKey, TransactionError};

mod emote_add;
mod emote_remove;
mod emote_update;

#[derive(Default)]
pub struct EmoteSetsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetsMutation {
	#[tracing::instrument(skip_all, name = "EmoteSetsMutation::emote_set")]
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<EmoteSetOps>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?;

		Ok(emote_set.map(|s| EmoteSetOps {
			id: s.id.into(),
			emote_set: s,
		}))
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetCreate, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetsMutation::create_emote_set")]
	async fn create_emote_set<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		user_id: GqlObjectId,
		data: CreateEmoteSetInput,
	) -> Result<EmoteSet, ApiError> {
		if data.privileged.unwrap_or(false) {
			return Err(ApiError::not_implemented(
				ApiErrorCode::BadRequest,
				"privileged emote sets are not supported",
			));
		}

		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let other_user = if user_id.id() == authed_user.id {
			None
		} else {
			Some(
				global
					.user_loader
					.load(global, user_id.id())
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?,
			)
		};

		let target = other_user.as_ref().unwrap_or(authed_user);

		if !target.has(EmoteSetPermission::Manage) && !authed_user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"this user does not have permission to create emote sets",
			));
		}

		if target.id != authed_user.id && !authed_user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: user_id.id(),
					editor_id: authed_user.id,
				})
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(ApiErrorCode::LackingPrivileges, "you are not an editor for this user")
				})?;

			if editor.state != UserEditorState::Accepted
				|| !editor.permissions.has_emote_set(EditorEmoteSetPermission::Create)
			{
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to create emote sets for this user",
				));
			}
		}

		let capacity = target.computed.permissions.emote_set_capacity.unwrap_or_default().max(0);

		if capacity == 0 {
			return Err(ApiError::bad_request(
				ApiErrorCode::LackingPrivileges,
				"maximum emote set capacity is 0, cannot create emote set",
			));
		}

		let res = transaction(global, |mut tx| async move {
			let emote_set_count = tx
				.count(
					filter::filter! {
						DbEmoteSet {
							owner_id: Some(user_id.id()),
						}
					},
					None,
				)
				.await?;

			if emote_set_count >= (target.computed.permissions.emote_set_limit.unwrap_or(0).max(0) as u64) {
				return Err(TransactionError::Custom(ApiError::bad_request(
					ApiErrorCode::LackingPrivileges,
					"maximum emote set limit reached",
				)));
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
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
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
	#[tracing::instrument(skip_all, name = "EmoteSetOps::check_perms")]
	async fn check_perms<'a>(
		&self,
		global: &Arc<Global>,
		session: &'a Session,
		editor_perm: impl Into<EditorPermission>,
	) -> Result<FullUserRef<'a>, ApiError> {
		let mut editor_perm = editor_perm.into();
		let user = session.user()?;

		let mut target = FullUserRef::Ref(user);

		match self.emote_set.kind {
			EmoteSetKind::Global => {
				if !user.has(EmoteSetPermission::ManageGlobal) {
					return Err(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"this user does not have permission to manage global emote sets",
					));
				}

				return Ok(target);
			}
			EmoteSetKind::Special => {
				if !user.has(EmoteSetPermission::ManageSpecial) {
					return Err(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
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
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "owner not found"))?;

		// If the person who is updating the set is not the owner, we need to load the
		// owner.
		if owner_id != user.id {
			target = FullUserRef::Owned(
				global
					.user_loader
					.load(global, owner_id)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "owner not found"))?
					.into(),
			)
		}

		let mut forbidden_msg = "you do not have permission to manage this user's emote sets";

		// If the emote set is personal, check if the owner has permission to use
		if matches!(self.emote_set.kind, EmoteSetKind::Personal) {
			if !target.has(UserPermission::UsePersonalEmoteSet) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"this user does not have permission to use personal emote sets",
				));
			}

			editor_perm = EditorUserPermission::ManagePersonalEmoteSet.into();
			forbidden_msg = "you do not have permission to manage this user's personal emote set";
		}

		if !target.has(EmoteSetPermission::Manage) && !user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, forbidden_msg));
		}

		if target.id != user.id && !user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: owner_id,
					editor_id: user.id,
				})
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(ApiErrorCode::LackingPrivileges, "you are not an editor for this user")
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(editor_perm) {
				return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, forbidden_msg));
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
	#[graphql(validator(minimum = 1))]
	capacity: Option<i32>,
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
	#[tracing::instrument(skip_all, name = "EmoteSetOps::emotes")]
	async fn emotes<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: GqlObjectId,
		action: ListItemAction,
		#[graphql(validator(custom = "EmoteNameValidator"))] name: Option<String>,
	) -> Result<Vec<ActiveEmote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.id.id()).into()),
			|tx| async move {
				match action {
					ListItemAction::Add => emote_add(global, tx, session, &self.emote_set, id.id(), name).await,
					ListItemAction::Remove => emote_remove(global, tx, session, &self.emote_set, id.id()).await,
					ListItemAction::Update => emote_update(global, tx, session, &self.emote_set, id.id(), name).await,
				}
			},
		)
		.await;

		match res {
			Ok(emote_set) => {
				let emotes = global
					.emote_by_id_loader
					.load_many_merged(emote_set.emotes.iter().map(|e| e.id))
					.await
					.map_err(|_| {
						ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to load emotes, however the operation was successful",
						)
					})?;

				Ok(emote_set
					.emotes
					.into_iter()
					.filter_map(|e| emotes.get(e.id).map(|emote| ActiveEmote::new(e, emote.clone())))
					.collect())
			}
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOps::update")]
	async fn update<'ctx>(&self, ctx: &Context<'ctx>, data: UpdateEmoteSetInput) -> Result<EmoteSet, ApiError> {
		// TODO: A bug in either the compiler or macro expansion, which causes the
		// linter to think we do not need a mutable `data` variable here.
		let mut data = data;

		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let sesison = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = sesison.user()?;

		let target = self.check_perms(global, sesison, EditorEmoteSetPermission::Manage).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.id.id()).into()),
			|mut tx| async move {
				let new_capacity = if let Some(capacity) = data.capacity {
					if capacity < self.emote_set.emotes.len() as i32 {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::BadRequest,
							"emote set capacity cannot be less than the number of emotes in the set",
						)));
					}

					let max_capacity = if self.emote_set.kind == EmoteSetKind::Personal {
						target.computed.permissions.personal_emote_set_capacity
					} else {
						target.computed.permissions.emote_set_capacity
					};

					if capacity > max_capacity.unwrap_or_default().max(0) {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::LackingPrivileges,
							"emote set capacity cannot exceed user's capacity",
						)));
					}

					Some(capacity)
				} else {
					None
				};

				if data.origins.is_some() {
					return Err(TransactionError::Custom(ApiError::not_implemented(
						ApiErrorCode::BadRequest,
						"legacy origins are not supported",
					)));
				}

				data.name.take_if(|n| n == &self.emote_set.name);

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
								search_updated_at: &None,
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to load emote set",
						))
					})?;

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
			},
		)
		.await;

		match res {
			Ok(emote_set) => Ok(EmoteSet::from_db(emote_set)),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOps::delete")]
	async fn delete<'ctx>(&self, ctx: &Context<'ctx>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		if matches!(
			self.emote_set.kind,
			EmoteSetKind::Personal | EmoteSetKind::Global | EmoteSetKind::Special
		) {
			return Err(ApiError::bad_request(
				ApiErrorCode::LackingPrivileges,
				"cannot delete personal, global, or special emote sets",
			));
		}

		let authed_user = session.user()?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.id.id()).into()),
			|mut tx| async move {
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
			},
		)
		.await;

		match res {
			Ok(deleted) => Ok(deleted),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}
}
