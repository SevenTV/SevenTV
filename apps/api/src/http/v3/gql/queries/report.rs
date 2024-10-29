use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use mongodb::bson::doc;
use shared::database::role::permissions::{RateLimitResource, TicketPermission};
use shared::database::ticket::{Ticket, TicketKind, TicketMemberKind, TicketMessage, TicketTarget};
use shared::old_types::object_id::GqlObjectId;

use super::user::{User, UserPartial};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::search::{search, sorted_results, SearchOptions};

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
		let Some(TicketTarget::Emote(emote_id)) = ticket.targets.first() else {
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
	#[tracing::instrument(skip_all, name = "Report::actor")]
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load_fast(global, self.actor_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user)
			.into())
	}

	#[tracing::instrument(skip_all, name = "Report::assignees")]
	async fn assignees<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<User>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load_fast_many(global, self.assignee_ids.iter().map(|i| i.id()))
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users"))?
			.into_values()
			.map(|u| UserPartial::from_db(global, u))
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
	#[graphql(
		guard = "PermissionGuard::one(TicketPermission::ManageAbuse).and(RateLimitGuard::new(RateLimitResource::Search, 1))"
	)]
	#[tracing::instrument(skip_all, name = "ReportsQuery::reports")]
	async fn reports<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		status: Option<ReportStatus>,
		limit: Option<u32>,
		page: Option<u32>,
	) -> Result<Vec<Report>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		if !matches!(status, Some(ReportStatus::Open) | None) {
			return Err(ApiError::not_implemented(
				ApiErrorCode::BadRequest,
				"only open reports are supported",
			));
		}

		let options = SearchOptions::builder()
			.query("*".to_owned())
			.filter_by(format!(
				"open: true && kind: {} && targets_emote: true",
				TicketKind::Abuse as i32
			))
			// Oldest highest priority first
			.sort_by(vec!["priority:desc".to_owned(), "created_at:asc".to_owned()])
			.page(page)
			.per_page(limit)
			.build();

		let result = search::<shared::typesense::types::ticket::Ticket>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let tickets = global
			.ticket_by_id_loader
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load tickets"))?;

		let mut messages = global
			.ticket_message_by_ticket_id_loader
			.load_many(tickets.keys().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load ticket messages"))?;

		Ok(sorted_results(result.hits, tickets)
			.into_iter()
			.filter_map(|ticket| {
				let messages = messages.remove(&ticket.id).unwrap_or_default();

				Report::from_db(ticket, messages)
			})
			.collect())
	}

	#[graphql(guard = "PermissionGuard::one(TicketPermission::ManageAbuse)")]
	#[tracing::instrument(skip_all, name = "ReportsQuery::report")]
	async fn report<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<Option<Report>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let Some(ticket) = global
			.ticket_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load ticket"))?
		else {
			return Ok(None);
		};

		if ticket.kind != TicketKind::Abuse && !matches!(ticket.targets.first(), Some(TicketTarget::Emote(_))) {
			return Ok(None);
		}

		let messages = global
			.ticket_message_by_ticket_id_loader
			.load(ticket.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load ticket messages"))?
			.unwrap_or_default();

		Ok(Report::from_db(ticket, messages))
	}
}
