use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use shared::database::{EmoteId, Id, TicketId, UserId};

use crate::http::error::ApiError;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/messages.gql

#[derive(Default)]
pub struct MessagesQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct InboxMessage {
    id: Id<()>,
    kind: MessageKind,
    created_at: chrono::DateTime<chrono::Utc>,
    author_id: Option<UserId>,
    read: bool,
    read_at: Option<chrono::DateTime<chrono::Utc>>,
    subject: String,
    content: String,
    important: bool,
    starred: bool,
    pinned: bool,
    placeholders: StringMap,
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct ModRequestMessage {
    id: TicketId,
    kind: MessageKind,
    // created_at
    author_id: Option<UserId>,
    read: bool,
    read_at: Option<chrono::DateTime<chrono::Utc>>,
    target_kind: u32,
    target_id: EmoteId,
    wish: String,
    actor_country_name: String,
    actor_country_code: String,
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl ModRequestMessage {
    async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.id.timestamp()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum MessageKind {
    EmoteComment,
    ModRequest,
    Inbox,
    News,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StringMap(async_graphql::indexmap::IndexMap<String, String>);

async_graphql::scalar!(StringMap);

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct ModRequestMessageList {
    messages: Vec<ModRequestMessage>,
    total: u32,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl MessagesQuery {
    async fn announcement<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<String>, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }

    async fn inbox<'ctx>(&self) -> Vec<InboxMessage> {
        // not implemented
        vec![]
    }

    async fn mod_requests<'ctx>(&self) -> Result<ModRequestMessageList, ApiError> {
        Err(ApiError::NOT_IMPLEMENTED)
    }
}
