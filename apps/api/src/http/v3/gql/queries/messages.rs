use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use futures::StreamExt;
use mongodb::bson::{doc, to_bson};
use mongodb::options::FindOptions;
use shared::database::{Collection, Ticket, TicketData, TicketMember, TicketMemberKind, TicketPermission, TicketStatus};
use shared::old_types::{EmoteObjectId, ObjectId, TicketObjectId, UserObjectId};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/messages.gql

#[derive(Default)]
pub struct MessagesQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct InboxMessage {
	id: ObjectId<()>,
	kind: MessageKind,
	created_at: chrono::DateTime<chrono::Utc>,
	author_id: Option<UserObjectId>,
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
	id: TicketObjectId,
	kind: MessageKind,
	// created_at
	author_id: Option<UserObjectId>,
	read: bool,
	read_at: Option<chrono::DateTime<chrono::Utc>>,
	target_kind: u32,
	target_id: EmoteObjectId,
	wish: String,
	actor_country_name: String,
	actor_country_code: String,
}

impl ModRequestMessage {
	fn from_db(ticket: Ticket, members: Vec<TicketMember>) -> Self {
		let op = members.iter().find(|m| m.kind == TicketMemberKind::Op);

		Self {
			id: ticket.id.into(),
			kind: MessageKind::ModRequest,
			author_id: op.map(|m| m.user_id.into()),
			read: ticket.status == TicketStatus::Fixed || ticket.status == TicketStatus::Closed,
			read_at: None,
			target_kind: 2,
			target_id: match ticket.data {
				TicketData::EmoteListingRequest { emote_id } => emote_id.into(),
				TicketData::EmotePersonalUseRequest { emote_id } => emote_id.into(),
				_ => EmoteObjectId::default(),
			},
			wish: match ticket.data {
				TicketData::EmoteListingRequest { .. } => "list".to_string(),
				TicketData::EmotePersonalUseRequest { .. } => "personal_use".to_string(),
				_ => String::new(),
			},
			actor_country_name: String::new(),
			actor_country_code: String::new(),
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl ModRequestMessage {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.id().timestamp()
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

	#[graphql(guard = "PermissionGuard::new(TicketPermission::Read)")]
	async fn mod_requests<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		after_id: Option<TicketObjectId>,
		limit: Option<u32>,
		wish: Option<String>,
		_country: Option<String>,
	) -> Result<ModRequestMessageList, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut search_args = mongodb::bson::Document::new();

		// only return open tickets?
		// not sure about this
		search_args.insert(
			"$or",
			vec![
				doc! { "status": to_bson(&TicketStatus::Pending).unwrap() },
				doc! { "status": to_bson(&TicketStatus::InProgress).unwrap() },
			],
		);

		match wish.as_ref().map(|s| s.as_str()) {
			Some("list") => {
				search_args.insert("data.kind", "emote_listing_request");
			}
			Some("personal_use") => {
				search_args.insert("data.kind", "emote_personal_use_request");
			}
			None => {}
			_ => return Err(ApiError::BAD_REQUEST),
		}

		let total = Ticket::collection(global.db())
			.count_documents(search_args.clone(), None)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if let Some(after_id) = after_id {
			search_args.insert("_id", doc! { "$gt": after_id.id() });
		}

		let limit = limit.unwrap_or(100).min(500);

		let tickets: Vec<_> = Ticket::collection(global.db())
			.find(
				search_args,
				FindOptions::builder().limit(limit as i64).sort(doc! { "_id": -1 }).build(),
			)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.filter_map(|r| async move {
				match r {
					Ok(ticket) => Some(ticket),
					Err(e) => {
						tracing::error!(error = %e, "failed to load ticket");
						None
					}
				}
			})
			.collect()
			.await;

		let mut all_members = global
			.ticket_members_by_ticket_id_loader()
			.load_many(tickets.iter().map(|t| t.id))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let messages = tickets
			.into_iter()
			.map(|t| {
				let members = all_members.remove(&t.id).unwrap_or_default();
				ModRequestMessage::from_db(t, members)
			})
			.collect();

		Ok(ModRequestMessageList { messages, total })
	}
}
