use async_graphql::Object;
use shared::old_types::{ObjectId, UserObjectId};

use crate::http::error::ApiError;
use crate::http::v3::gql::queries::InboxMessage;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/resolvers/mutation/mutation.messages.go

#[derive(Default)]
pub struct MessagesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl MessagesMutation {
	async fn read_messages(&self, _message_ids: Vec<ObjectId<()>>, _read: bool) -> Result<u32, ApiError> {
		// will be left unimplemented
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn send_inbox_message(
		&self,
		_recipients: Vec<ObjectId<UserObjectId>>,
		_subject: String,
		_content: String,
		_important: Option<bool>,
		_anonymous: Option<bool>,
	) -> Result<Option<InboxMessage>, ApiError> {
		// will be left unimplemented
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn dismiss_void_target_mod_requests(&self, _object: u32) -> Result<u32, ApiError> {
		// will be left unimplemented
		Err(ApiError::NOT_IMPLEMENTED)
	}
}
