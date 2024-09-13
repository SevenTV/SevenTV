use std::sync::Arc;

use async_graphql::{Context, Object};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestId, EmoteModerationRequestStatus,
};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::EmoteModerationRequestPermission;
use shared::database::MongoCollection;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::message::InboxMessage;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/resolvers/mutation/mutation.messages.go

#[derive(Default)]
pub struct MessagesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl MessagesMutation {
	#[graphql(guard = "PermissionGuard::one(EmoteModerationRequestPermission::Manage)")]
	async fn read_messages<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		message_ids: Vec<GqlObjectId>,
		read: bool,
		approved: bool,
	) -> Result<u32, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let ids: Vec<EmoteModerationRequestId> = message_ids.into_iter().map(|id| id.id()).collect();

		let status = if read && approved {
			EmoteModerationRequestStatus::Approved
		} else if read {
			EmoteModerationRequestStatus::Denied
		} else {
			EmoteModerationRequestStatus::Pending
		};

		// TODO: events?
		let res = EmoteModerationRequest::collection(&global.db)
			.update_many(
				filter::filter! {
					EmoteModerationRequest {
						#[query(rename = "_id", selector = "in")]
						id: ids,
					}
				},
				update::update! {
					#[query(set)]
					EmoteModerationRequest {
						#[query(serde)]
						status: status,
					}
				},
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update moderation requests");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update moderation requests")
			})?;

		Ok(res.modified_count as u32)
	}

	async fn send_inbox_message(
		&self,
		_recipients: Vec<GqlObjectId>,
		_subject: String,
		_content: String,
		_important: Option<bool>,
		_anonymous: Option<bool>,
	) -> Result<Option<InboxMessage>, ApiError> {
		// will be left unimplemented
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	async fn dismiss_void_target_mod_requests(&self, _object: u32) -> Result<u32, ApiError> {
		// will be left unimplemented
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}
}
