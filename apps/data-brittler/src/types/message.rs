use mongodb::bson::oid::ObjectId;

#[derive(Debug, serde::Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub kind: MessageKind,
    pub author_id: ObjectId,
    pub created_at: super::DateTime,
    pub data: MessageData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u32)]
pub enum MessageKind {
    EmoteComment = 1,
    ModRequest = 2,
    Inbox = 3,
    News = 4,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum MessageData {
    InboxData {
        subject: String,
        content: String,
    },
    EmoteRequest {
        target_id: ObjectId,
        wish: Option<EmoteWish>,
    },
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmoteWish {
    List,
    PersonalUse,
}

#[derive(Debug, serde::Deserialize)]
pub struct MessageRead {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub message_id: ObjectId,
    pub read: bool,
}
