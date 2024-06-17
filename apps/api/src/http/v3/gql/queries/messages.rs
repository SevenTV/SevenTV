use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use shared::database::emote_moderation_request::{EmoteModerationRequest, EmoteModerationRequestKind, EmoteModerationRequestStatus};
use shared::old_types::object_id::GqlObjectId;
use shared::database::role::permissions::EmoteModerationRequestPermission;
use shared::database::Collection;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/messages.gql

#[derive(Default)]
pub struct MessagesQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct InboxMessage {
	id: GqlObjectId,
	kind: MessageKind,
	created_at: chrono::DateTime<chrono::Utc>,
	author_id: Option<GqlObjectId>,
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
	id: GqlObjectId,
	kind: MessageKind,
	// created_at
	author_id: Option<GqlObjectId>,
	read: bool,
	read_at: Option<chrono::DateTime<chrono::Utc>>,
	target_kind: u32,
	target_id: GqlObjectId,
	wish: String,
	actor_country_name: String,
	actor_country_code: String,
}

impl ModRequestMessage {
	fn from_db(mod_request: EmoteModerationRequest) -> Self {
		Self {
			id: mod_request.id.into(),
			kind: MessageKind::ModRequest,
			author_id: Some(mod_request.user_id.into()),
			read: mod_request.status == EmoteModerationRequestStatus::Approved || mod_request.status == EmoteModerationRequestStatus::Denied,
			read_at: None,
			target_kind: 2,
			target_id: mod_request.emote_id.into(),
			wish: match mod_request.kind {
				EmoteModerationRequestKind::PublicListing => "list".to_string(),
				EmoteModerationRequestKind::PersonalUse => "personal_use".to_string(),
			},
			actor_country_name: String::new(),
			actor_country_code: mod_request.country_code.unwrap_or_default(),
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl ModRequestMessage {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
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
	total: u64,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl MessagesQuery {
	async fn announcement<'ctx>(&self, ctx: &Context<'ctx>) -> Result<String, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let message = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?
			.alerts
			.message;

		Ok(message.unwrap_or_default())
	}

	async fn inbox<'ctx>(&self) -> Vec<InboxMessage> {
		// not implemented
		vec![]
	}

	#[graphql(guard = "PermissionGuard::one(EmoteModerationRequestPermission::Manage)")]
	async fn mod_requests<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		after_id: Option<GqlObjectId>,
		limit: Option<u32>,
		wish: Option<String>,
		_country: Option<String>,
	) -> Result<ModRequestMessageList, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut search_args = mongodb::bson::Document::new();

		// only return open tickets?
		// not sure about this
		search_args.insert("status", EmoteModerationRequestStatus::Pending as i32);

		match wish.as_ref().map(|s| s.as_str()) {
			Some("list") => {
				search_args.insert("kind", EmoteModerationRequestKind::PublicListing as i32);
			}
			Some("personal_use") => {
				search_args.insert("kind", EmoteModerationRequestKind::PersonalUse as i32);
			}
			None => {}
			_ => return Err(ApiError::BAD_REQUEST),
		}

		let total = EmoteModerationRequest::collection(global.db())
			.count_documents(search_args.clone(), None)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if let Some(after_id) = after_id {
			search_args.insert("_id", doc! { "$gt": after_id.0 });
		}

		let limit = limit.unwrap_or(100).min(500);

		let mod_requests: Vec<_> = EmoteModerationRequest::collection(global.db())
			.find(
				search_args,
				FindOptions::builder().limit(limit as i64).sort(doc! { "_id": -1 }).build(),
			)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.filter_map(|r| async move {
				match r {
					Ok(mod_request) => Some(mod_request),
					Err(e) => {
						tracing::error!(error = %e, "failed to load mod request");
						None
					}
				}
			})
			.collect()
			.await;

		let messages = mod_requests.into_iter().map(ModRequestMessage::from_db).collect();

		Ok(ModRequestMessageList { messages, total })
	}
}
