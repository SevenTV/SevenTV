use std::collections::HashMap;

use crate::database::{Id, UserId};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/message.model.go#L9
pub struct InboxMessageModel {
	pub id: Id,
	pub kind: MessageKind,
	#[serde(rename = "createdAt")]
	pub created_at: i64,
	pub author_id: Option<UserId>,
	pub read: bool,
	#[serde(rename = "readAt")]
	pub read_at: Option<i64>,
	pub subject: String,
	pub content: String,
	pub important: bool,
	pub starred: bool,
	pub pinned: bool,
	pub placeholders: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/message.model.go#L24
pub struct ModRequestMessageModel {
	pub id: Id,
	pub kind: MessageKind,
	#[serde(rename = "createdAt")]
	pub created_at: i64,
	pub author_id: Option<UserId>,
	#[serde(rename = "targetKind")]
	pub target_kind: i32,
	#[serde(rename = "targetID")]
	pub target_id: UserId,
	pub read: bool,
	pub wish: String,
	pub actor_country_name: String,
	pub actor_country_code: String,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/message.model.go#L37
pub enum MessageKind {
	#[default]
	EmoteComment,
	ModRequest,
	Inbox,
	News,
}
