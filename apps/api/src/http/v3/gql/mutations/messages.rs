use std::sync::Arc;

use async_graphql::{Context, Object};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestId, EmoteModerationRequestStatus,
};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::EmoteModerationRequestPermission;
use shared::database::stored_event::StoredEventEmoteModerationRequestData;
use shared::event::{InternalEvent, InternalEventData};
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::message::InboxMessage;
use crate::transactions::{with_transaction, TransactionError};

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
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let ids: Vec<EmoteModerationRequestId> = message_ids.into_iter().map(|id| id.id()).collect();

		let status = if read && approved {
			EmoteModerationRequestStatus::Approved
		} else if read {
			EmoteModerationRequestStatus::Denied
		} else {
			EmoteModerationRequestStatus::Pending
		};

		let res = with_transaction(global, |mut tx| async move {
			let requests = tx
				.find(
					filter::filter! {
						EmoteModerationRequest {
							#[query(rename = "_id", selector = "in")]
							id: &ids,
						}
					},
					None,
				)
				.await?;

			let res = tx
				.update(
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
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await?;

			if res.modified_count != requests.len() as u64 {
				return Err(TransactionError::Custom(ApiError::not_found(
					ApiErrorCode::LoadError,
					"at least one message was not found",
				)));
			}

			for req in requests {
				let old = req.status;

				let mut after = req;
				after.status = status;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::EmoteModerationRequest {
						after,
						data: StoredEventEmoteModerationRequestData::ChangeStatus { old, new: status },
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			Ok(res.modified_count as u32)
		})
		.await;

		match res {
			Ok(res) => Ok(res),
			Err(e) => {
				tracing::error!(error = %e, "failed to update moderation requests");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"failed to update moderation requests",
				))
			}
		}
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
