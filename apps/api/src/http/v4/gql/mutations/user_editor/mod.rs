use std::sync::Arc;

use async_graphql::Context;
use shared::database::role::permissions::{FlagPermission, PermissionsExt, UserPermission};
use shared::database::user::editor::{
	EditorEmotePermission, EditorEmoteSetPermission, EditorUserPermission, UserEditorId, UserEditorPermissions,
	UserEditorState,
};
use shared::database::user::UserId;
use shared::event::{InternalEvent, InternalEventData, InternalEventUserEditorData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::UserEditor;
use crate::transactions::{transaction_with_mutex, GeneralMutexKey, TransactionError};

mod operation;

#[derive(Default)]
pub struct UserEditorMutation;

#[async_graphql::Object]
impl UserEditorMutation {
	#[tracing::instrument(skip_all, name = "UserEditorMutation::editor")]
	async fn editor(
		&self,
		ctx: &Context<'_>,
		user_id: UserId,
		editor_id: UserId,
	) -> Result<operation::UserEditorOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user_editor = global
			.user_editor_by_id_loader
			.load(UserEditorId { user_id, editor_id })
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user editor"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user editor not found"))?;

		Ok(operation::UserEditorOperation { user_editor })
	}

	#[tracing::instrument(skip_all, name = "UserEditorMutation::create")]
	async fn create(
		&self,
		ctx: &Context<'_>,
		user_id: UserId,
		editor_id: UserId,
		permissions: UserEditorPermissionsInput,
	) -> Result<UserEditor, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if !authed_user.has(UserPermission::InviteEditors) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you are not allowed to invite editors",
			));
		}

		let permissions: UserEditorPermissions = permissions.into();

		if authed_user.id != user_id && !authed_user.has(UserPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id,
				})
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"you do not have permission to modify editors",
					)
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(EditorUserPermission::ManageEditors) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to modify editors, you need the ManageEditors permission",
				));
			}

			if permissions.is_superset_of(&editor.permissions) {
				return Err(ApiError::bad_request(
					ApiErrorCode::BadRequest,
					"you cannot grant permissions that you do not have",
				));
			}
		}

		if user_id == editor_id {
			return Err(ApiError::bad_request(
				ApiErrorCode::BadRequest,
				"you cannot invite yourself as an editor",
			));
		}

		let editors = global
			.user_editor_by_user_id_loader
			.load(user_id)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
			.unwrap_or_default();

		if editors.iter().any(|editor| editor.id.editor_id == editor_id) {
			return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "editor already exists"));
		}

		let res = transaction_with_mutex(global, Some(GeneralMutexKey::User(user_id).into()), |mut tx| async move {
			let editor_user = global
				.user_loader
				.load_fast(global, editor_id)
				.await
				.map_err(|_| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::LoadError,
						"failed to load user",
					))
				})?
				.ok_or_else(|| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::LoadError,
						"failed to load user",
					))
				})?;

			let state = if editor_user.has(FlagPermission::InstantInvite) {
				UserEditorState::Accepted
			} else {
				UserEditorState::Pending
			};

			let editor = shared::database::user::editor::UserEditor {
				id: UserEditorId { user_id, editor_id },
				permissions,
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
				state,
				notes: None,
				added_at: chrono::Utc::now(),
				added_by_id: authed_user.id,
			};

			tx.insert_one::<shared::database::user::editor::UserEditor>(&editor, None)
				.await?;

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::UserEditor {
					after: editor.clone(),
					data: InternalEventUserEditorData::AddEditor {
						editor: Box::new(editor_user.user),
					},
				},
				timestamp: chrono::Utc::now(),
			})?;

			Ok(editor)
		})
		.await;

		match res {
			Ok(editor) => Ok(editor.into()),
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

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct UserEditorPermissionsInput {
	pub super_admin: bool,
	pub emote_set: EditorEmoteSetPermissionInput,
	pub emote: EditorEmotePermissionInput,
	pub user: EditorUserPermissionInput,
}

impl From<UserEditorPermissionsInput> for UserEditorPermissions {
	fn from(value: UserEditorPermissionsInput) -> Self {
		let mut user = value.user.into();

		if value.super_admin {
			user |= EditorUserPermission::SuperAdmin;
		}

		Self {
			user,
			emote_set: value.emote_set.into(),
			emote: value.emote.into(),
		}
	}
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct EditorEmoteSetPermissionInput {
	pub admin: bool,
	pub manage: bool,
	pub create: bool,
}

impl From<EditorEmoteSetPermissionInput> for EditorEmoteSetPermission {
	fn from(value: EditorEmoteSetPermissionInput) -> EditorEmoteSetPermission {
		let mut perms = EditorEmoteSetPermission::default();

		if value.admin {
			perms |= EditorEmoteSetPermission::Admin;
		}

		if value.manage {
			perms |= EditorEmoteSetPermission::Manage;
		}

		if value.create {
			perms |= EditorEmoteSetPermission::Create;
		}

		perms
	}
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct EditorEmotePermissionInput {
	pub admin: bool,
	pub manage: bool,
	pub create: bool,
	pub transfer: bool,
}

impl From<EditorEmotePermissionInput> for EditorEmotePermission {
	fn from(value: EditorEmotePermissionInput) -> EditorEmotePermission {
		let mut perms = EditorEmotePermission::default();

		if value.admin {
			perms |= EditorEmotePermission::Admin;
		}

		if value.manage {
			perms |= EditorEmotePermission::Manage;
		}

		if value.create {
			perms |= EditorEmotePermission::Create;
		}

		if value.transfer {
			perms |= EditorEmotePermission::Transfer;
		}

		perms
	}
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct EditorUserPermissionInput {
	pub admin: bool,
	pub manage_billing: bool,
	pub manage_profile: bool,
	pub manage_editors: bool,
	pub manage_personal_emote_set: bool,
}

impl From<EditorUserPermissionInput> for EditorUserPermission {
	fn from(value: EditorUserPermissionInput) -> EditorUserPermission {
		let mut perms = EditorUserPermission::default();

		if value.admin {
			perms |= EditorUserPermission::Admin;
		}

		if value.manage_billing {
			perms |= EditorUserPermission::ManageBilling;
		}

		if value.manage_profile {
			perms |= EditorUserPermission::ManageProfile;
		}

		if value.manage_editors {
			perms |= EditorUserPermission::ManageEditors;
		}

		if value.manage_personal_emote_set {
			perms |= EditorUserPermission::ManagePersonalEmoteSet;
		}

		perms
	}
}
