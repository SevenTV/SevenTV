use std::{ops::Deref, sync::Arc};

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use futures::StreamExt;
use mongodb::{
	bson::{doc, to_bson},
	options::FindOptions,
};
use shared::database::{
	Collection, Ticket, TicketData, TicketMember, TicketMemberKind, TicketMessage, TicketPermission, TicketStatus,
};

use crate::http::v3::gql::{
	guards::PermissionGuard,
	object_id::{EmoteObjectId, TicketObjectId, UserObjectId},
};
use crate::{global::Global, http::error::ApiError};

use super::users::{User, UserPartial};

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/reports.gql

#[derive(Default)]
pub struct ReportsQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Report {
	id: TicketObjectId,
	target_kind: u32,
	target_id: EmoteObjectId,
	actor_id: UserObjectId,
	// actor
	subject: String,
	body: String,
	priority: u8,
	status: ReportStatus,
	// created_at
	notes: Vec<String>,
	// assignees
	#[graphql(skip)]
	assignee_ids: Vec<UserObjectId>,
}

impl Report {
	fn from_db(ticket: Ticket, members: Vec<TicketMember>, messages: Vec<TicketMessage>) -> Option<Self> {
		let TicketData::EmoteReport { emote_id } = ticket.data else {
			return None;
		};

		let actor_id = members.iter().find(|m| m.kind == TicketMemberKind::Op)?.user_id;

		let mut op_messages: Vec<_> = messages.iter().filter(|m| m.user_id == actor_id).collect();
		op_messages.sort_by_key(|m| m.id);

		let body_msg = op_messages.first();

		let notes = messages
			.iter()
			.filter(|m| Some(m.id) != body_msg.map(|b| b.id))
			.map(|m| m.content.clone())
			.collect();

		let assignee_ids = members
			.iter()
			.filter(|m| m.kind == TicketMemberKind::Staff)
			.map(|m| m.user_id.into())
			.collect();

		Some(Self {
			id: ticket.id.into(),
			target_kind: 2,
			target_id: emote_id.into(),
			actor_id: actor_id.into(),
			subject: ticket.title,
			body: body_msg.map(|m| m.content.clone()).unwrap_or_default(),
			priority: ticket.priority as u8,
			status: ticket.status.into(),
			notes,
			assignee_ids,
		})
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Report {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User, ApiError> {
		let global = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(UserPartial::load_from_db(global, *self.actor_id).await?.into())
	}

	async fn assignees<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<User>, ApiError> {
		let global = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(
			UserPartial::load_many_from_db(global, self.assignee_ids.iter().map(|i| i.deref().clone()))
				.await?
				.into_iter()
				.map(Into::into)
				.collect(),
		)
	}

	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.timestamp()
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ReportStatus {
	Open,
	Assigned,
	Closed,
}

impl From<TicketStatus> for ReportStatus {
	fn from(value: TicketStatus) -> Self {
		match value {
			TicketStatus::Pending => Self::Open,
			TicketStatus::InProgress => Self::Open,
			TicketStatus::Fixed => Self::Closed,
			TicketStatus::Closed => Self::Closed,
		}
	}
}

impl From<ReportStatus> for TicketStatus {
	fn from(value: ReportStatus) -> Self {
		match value {
			ReportStatus::Open => TicketStatus::Pending,
			ReportStatus::Assigned => TicketStatus::InProgress,
			ReportStatus::Closed => TicketStatus::Fixed,
		}
	}
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl ReportsQuery {
	#[graphql(guard = "PermissionGuard::new(TicketPermission::Read)")]
	async fn reports<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		status: Option<ReportStatus>,
		limit: Option<u32>,
		after_id: Option<TicketObjectId>,
		before_id: Option<TicketObjectId>,
	) -> Result<Vec<Report>, ApiError> {
		if let (Some(after_id), Some(before_id)) = (after_id.map(|i| *i), before_id.map(|i| *i)) {
			if after_id > before_id {
				return Err(ApiError::BAD_REQUEST);
			}
		}

		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let limit = limit.unwrap_or(12).min(100);

		let mut search_args = mongodb::bson::Document::new();
		// only return emote reports
		search_args.insert("data.kind", "emote_report");

		let mut id_args = mongodb::bson::Document::new();

		if let Some(after_id) = after_id {
			id_args.insert("$gt", *after_id);
		}

		if let Some(before_id) = before_id {
			id_args.insert("$lt", *before_id);
		}

		if id_args.len() > 0 {
			search_args.insert("_id", id_args);
		}

		if let Some(status) = status {
			search_args.insert(
				"status",
				to_bson(&TicketStatus::from(status)).map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?,
			);
		}

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

		let mut all_messages = global
			.ticket_messages_by_ticket_id_loader()
			.load_many(tickets.iter().map(|t| t.id))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(tickets
			.into_iter()
			.filter_map(|t| {
				let members = all_members.remove(&t.id).unwrap_or_default();
				let messages = all_messages.remove(&t.id).unwrap_or_default();
				Report::from_db(t, members, messages)
			})
			.collect())
	}

	#[graphql(guard = "PermissionGuard::new(TicketPermission::Read)")]
	async fn report<'ctx>(&self, ctx: &Context<'ctx>, id: TicketObjectId) -> Result<Option<Report>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let Some(ticket) = global
			.ticket_by_id_loader()
			.load(*id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		else {
			return Ok(None);
		};

		let members = global
			.ticket_members_by_ticket_id_loader()
			.load(*id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		let messages = global
			.ticket_messages_by_ticket_id_loader()
			.load(*id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(Report::from_db(ticket, members, messages))
	}
}
