use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum};
use shared::database::user::UserId;

use super::User;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct UserEditor {
	pub user_id: UserId,
	pub editor_id: UserId,
	pub state: UserEditorState,
	pub notes: Option<String>,
	pub permissions: UserEditorPermissions,
	pub added_by_id: UserId,
	pub added_at: chrono::DateTime<chrono::Utc>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<shared::database::user::editor::UserEditor> for UserEditor {
	fn from(value: shared::database::user::editor::UserEditor) -> Self {
		Self {
			user_id: value.id.user_id,
			editor_id: value.id.editor_id,
			state: value.state.into(),
			notes: value.notes,
			permissions: value.permissions.into(),
			added_by_id: value.added_by_id,
			added_at: value.added_at,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[ComplexObject]
impl UserEditor {
	#[tracing::instrument(skip_all, name = "UserEditor::editor")]
	async fn editor(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.editor_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "UserEditor::user")]
	async fn user(&self, ctx: &Context<'_>) -> Result<Option<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, self.user_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(user.map(Into::into))
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
pub enum UserEditorState {
	Pending,
	Accepted,
	Rejected,
}

impl From<shared::database::user::editor::UserEditorState> for UserEditorState {
	fn from(value: shared::database::user::editor::UserEditorState) -> Self {
		match value {
			shared::database::user::editor::UserEditorState::Pending => Self::Pending,
			shared::database::user::editor::UserEditorState::Accepted => Self::Accepted,
			shared::database::user::editor::UserEditorState::Rejected => Self::Rejected,
		}
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct UserEditorPermissions {
	pub super_admin: bool,
	pub emote_set: EditorEmoteSetPermission,
	pub emote: EditorEmotePermission,
	pub user: EditorUserPermission,
}

impl From<shared::database::user::editor::UserEditorPermissions> for UserEditorPermissions {
	fn from(value: shared::database::user::editor::UserEditorPermissions) -> Self {
		Self {
			super_admin: value.has_user(shared::database::user::editor::EditorUserPermission::SuperAdmin),
			emote_set: EditorEmoteSetPermission::from_db(&value),
			emote: EditorEmotePermission::from_db(&value),
			user: EditorUserPermission::from_db(&value),
		}
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct EditorEmoteSetPermission {
	pub admin: bool,
	pub manage: bool,
	pub create: bool,
}

impl EditorEmoteSetPermission {
	fn from_db(value: &shared::database::user::editor::UserEditorPermissions) -> Self {
		Self {
			admin: value.has_emote_set(shared::database::user::editor::EditorEmoteSetPermission::Admin),
			manage: value.has_emote_set(shared::database::user::editor::EditorEmoteSetPermission::Manage),
			create: value.has_emote_set(shared::database::user::editor::EditorEmoteSetPermission::Create),
		}
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct EditorEmotePermission {
	pub admin: bool,
	pub manage: bool,
	pub create: bool,
	pub transfer: bool,
}

impl EditorEmotePermission {
	fn from_db(value: &shared::database::user::editor::UserEditorPermissions) -> Self {
		Self {
			admin: value.has_emote(shared::database::user::editor::EditorEmotePermission::Admin),
			manage: value.has_emote(shared::database::user::editor::EditorEmotePermission::Manage),
			create: value.has_emote(shared::database::user::editor::EditorEmotePermission::Create),
			transfer: value.has_emote(shared::database::user::editor::EditorEmotePermission::Transfer),
		}
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
pub struct EditorUserPermission {
	pub admin: bool,
	pub manage_billing: bool,
	pub manage_profile: bool,
	pub manage_editors: bool,
	pub manage_personal_emote_set: bool,
}

impl EditorUserPermission {
	fn from_db(value: &shared::database::user::editor::UserEditorPermissions) -> Self {
		Self {
			admin: value.has_user(shared::database::user::editor::EditorUserPermission::Admin),
			manage_billing: value.has_user(shared::database::user::editor::EditorUserPermission::ManageBilling),
			manage_profile: value.has_user(shared::database::user::editor::EditorUserPermission::ManageProfile),
			manage_editors: value.has_user(shared::database::user::editor::EditorUserPermission::ManageEditors),
			manage_personal_emote_set: value
				.has_user(shared::database::user::editor::EditorUserPermission::ManagePersonalEmoteSet),
		}
	}
}
