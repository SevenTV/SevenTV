use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use shared::database::user::UserId;

use super::User;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct UserEditor {
	pub user_id: UserId,
	pub editor_id: UserId,
	pub state: UserEditorState,
	pub notes: Option<String>,
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
			added_by_id: value.added_by_id,
			added_at: value.added_at,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[ComplexObject]
impl UserEditor {
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
