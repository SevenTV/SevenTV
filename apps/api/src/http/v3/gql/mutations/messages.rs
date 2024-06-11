use async_graphql::Object;
use shared::old_types::{ObjectId, UserObjectId};

use crate::http::error::ApiError;
use crate::http::v3::gql::queries::InboxMessage;

#[derive(Default)]
pub struct MessagesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl MessagesMutation {
	async fn read_messages(&self, message_ids: Vec<ObjectId<()>>, read: bool) -> Result<u32, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn send_inbox_message(
		&self,
		recipients: Vec<ObjectId<UserObjectId>>,
		subject: String,
		content: String,
		important: Option<bool>,
		anonymous: Option<bool>,
	) -> Result<Option<InboxMessage>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn dismiss_void_target_mod_requests(&self, object: u32) -> Result<u32, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}
}
