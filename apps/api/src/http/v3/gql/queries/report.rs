use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use shared::database::role::permissions::TicketPermission;
use shared::database::ticket::{Ticket, TicketKind, TicketMemberKind, TicketMessage, TicketTarget};
use shared::database::Collection;
use shared::old_types::object_id::GqlObjectId;

use super::user::{User, UserPartial};
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/reports.gql

#[derive(Default)]
pub struct ReportsQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Report {
	id: GqlObjectId,
	target_kind: u32,
	target_id: GqlObjectId,
	actor_id: GqlObjectId,
	// actor
	subject: String,
	body: String,
	priority: i32,
	status: ReportStatus,
	// created_at
	notes: Vec<String>,
	// assignees
	#[graphql(skip)]
	assignee_ids: Vec<GqlObjectId>,
}

impl Report {
	pub fn from_db(ticket: Ticket, messages: Vec<TicketMessage>) -> Option<Self> {
		let Some(TicketTarget::Emote(emote_id)) = ticket.targets.get(0) else {
			return None;
		};

		let mut op_messages: Vec<_> = messages.iter().filter(|m| m.user_id == ticket.author_id).collect();
		op_messages.sort_by_key(|m| m.id);

		let body_msg = op_messages.first();

		let status = if ticket.open {
			if ticket.members.iter().any(|m| m.kind == TicketMemberKind::Assigned) {
				ReportStatus::Assigned
			} else {
				ReportStatus::Open
			}
		} else {
			ReportStatus::Closed
		};

		let notes = messages
			.iter()
			.filter(|m| Some(m.id) != body_msg.map(|b| b.id))
			.map(|m| m.content.clone())
			.collect();

		let assignee_ids = ticket
			.members
			.iter()
			.filter(|m| m.kind == TicketMemberKind::Assigned)
			.map(|m| m.user_id.into())
			.collect();

		Some(Self {
			id: ticket.id.into(),
			target_kind: 2,
			target_id: (*emote_id).into(),
			actor_id: ticket.author_id.into(),
			subject: ticket.title,
			body: body_msg.map(|m| m.content.clone()).unwrap_or_default(),
			priority: ticket.priority as i32,
			status,
			notes,
			assignee_ids,
		})
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Report {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.user_by_id_loader()
			.load(self.actor_id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u.into()))
			.unwrap_or_else(UserPartial::deleted_user)
			.into())
	}

	async fn assignees<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<User>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.user_by_id_loader()
			.load_many(self.assignee_ids.iter().map(|i| i.0.cast()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.map(|u| UserPartial::from_db(global, u.into()))
			.map(Into::into)
			.collect())
	}

	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, serde::Deserialize, serde::Serialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportStatus {
	Open,
	Assigned,
	Closed,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl ReportsQuery {
	#[graphql(guard = "PermissionGuard::one(TicketPermission::ManageAbuse)")]
	async fn reports<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		status: Option<ReportStatus>,
		limit: Option<u32>,
		after_id: Option<GqlObjectId>,
		before_id: Option<GqlObjectId>,
	) -> Result<Vec<Report>, ApiError> {
		if let (Some(after_id), Some(before_id)) = (after_id.map(|i| i.0), before_id.map(|i| i.0)) {
			if after_id > before_id {
				return Err(ApiError::BAD_REQUEST);
			}
		}

		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let limit = limit.unwrap_or(12).min(100);

		let mut search_args = mongodb::bson::Document::new();
		// only return emote reports
		search_args.insert("kind", TicketKind::Abuse as i32);
		search_args.insert("targets.kind", "emote");

		let mut id_args = mongodb::bson::Document::new();

		if let Some(after_id) = after_id {
			id_args.insert("$gt", after_id.0);
		}

		if let Some(before_id) = before_id {
			id_args.insert("$lt", before_id.0);
		}

		if id_args.len() > 0 {
			search_args.insert("_id", id_args);
		}

		match status {
			Some(ReportStatus::Open) => {
				search_args.insert("open", true);
			}
			Some(ReportStatus::Closed) => {
				search_args.insert("open", false);
			}
			_ => {}
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

		let mut all_messages = global
			.ticket_messages_by_ticket_id_loader()
			.load_many(tickets.iter().map(|t| t.id))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(tickets
			.into_iter()
			.filter_map(|t| {
				let messages = all_messages.remove(&t.id).unwrap_or_default();
				Report::from_db(t, messages)
			})
			.collect())
	}

	#[graphql(guard = "PermissionGuard::one(TicketPermission::ManageAbuse)")]
	async fn report<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<Report>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let Some(ticket) = Ticket::collection(global.db())
			.find_one(
				doc! {
					"_id": id.0,
					"kind": TicketKind::Abuse as i32,
					"targets.kind": "emote",
				},
				None,
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to load ticket");
				ApiError::INTERNAL_SERVER_ERROR
			})?
		else {
			return Ok(None);
		};

		let messages = global
			.ticket_messages_by_ticket_id_loader()
			.load(id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(Report::from_db(ticket, messages))
	}
}
