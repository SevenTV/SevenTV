use std::sync::Arc;

use async_graphql::Context;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, UserPermission};
use shared::database::user::editor::{EditorUserPermission, UserEditorId, UserEditorState};
use shared::event::{InternalEvent, InternalEventData, InternalEventUserEditorData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::UserEditor;
use crate::transactions::{transaction_with_mutex, GeneralMutexKey, TransactionError};

pub struct UserEditorOperation {
	pub user_editor: shared::database::user::editor::UserEditor,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum UserEditorUpdateState {
	Accept,
	Reject,
}

impl From<UserEditorUpdateState> for shared::database::user::editor::UserEditorState {
	fn from(state: UserEditorUpdateState) -> Self {
		match state {
			UserEditorUpdateState::Accept => shared::database::user::editor::UserEditorState::Accepted,
			UserEditorUpdateState::Reject => shared::database::user::editor::UserEditorState::Rejected,
		}
	}
}

#[async_graphql::Object]
impl UserEditorOperation {
	#[tracing::instrument(skip_all, name = "UserEditorOperation::delete")]
	async fn delete(&self, ctx: &Context<'_>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if self.user_editor.state == UserEditorState::Rejected && authed_user.id != self.user_editor.id.editor_id {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you do not have permission to delete rejected editors",
			));
		}

		// They should be able to remove themselves from the editor list
		if authed_user.id != self.user_editor.id.user_id
			&& !authed_user.has(UserPermission::ManageAny)
			&& authed_user.id != self.user_editor.id.editor_id
		{
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id: self.user_editor.id.user_id,
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
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::User(self.user_editor.id.user_id).into()),
			|mut tx| async move {
				// Remove editor
				let res = tx
					.find_one_and_delete(
						filter::filter! {
							shared::database::user::editor::UserEditor {
								#[query(rename = "_id", serde)]
								id: self.user_editor.id,
							}
						},
						None,
					)
					.await?;

				if let Some(editor) = res {
					let editor_user = global
						.user_loader
						.load_fast(global, editor.id.editor_id)
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

					tx.register_event(InternalEvent {
						actor: Some(authed_user.clone()),
						session_id: session.user_session_id(),
						data: InternalEventData::UserEditor {
							after: editor,
							data: InternalEventUserEditorData::RemoveEditor {
								editor: Box::new(editor_user.user),
							},
						},
						timestamp: chrono::Utc::now(),
					})?;

					Ok(true)
				} else {
					Ok(false)
				}
			},
		)
		.await;

		match res {
			Ok(res) => Ok(res),
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

	#[tracing::instrument(skip_all, name = "UserEditorOperation::update_state")]
	async fn update_state(&self, ctx: &Context<'_>, state: UserEditorUpdateState) -> Result<UserEditor, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.user_editor.id.editor_id {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you do not have permission to update editor state on behalf of others",
			));
		}

		if self.user_editor.state != UserEditorState::Pending {
			return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "editor is not pending"));
		}

		let state = shared::database::user::editor::UserEditorState::from(state);

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::User(self.user_editor.id.user_id).into()),
			|mut tx| async move {
				let editor = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::user::editor::UserEditor {
								#[query(serde, rename = "_id")]
								id: self.user_editor.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::user::editor::UserEditor {
								#[query(serde)]
								state,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to update editor",
						))
					})?;

				Ok(editor)
			},
		)
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

	#[tracing::instrument(skip_all, name = "UserEditorOperation::update_permissions")]
	async fn update_permissions(&self, _ctx: &Context<'_>) -> Result<UserEditor, ApiError> {
		todo!()
	}
}
