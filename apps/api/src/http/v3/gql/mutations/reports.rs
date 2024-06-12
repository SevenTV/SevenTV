use std::str::FromStr;
use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::{self, Collection, TicketMember, TicketMemberKind, TicketPermission, TicketStatus, UserId};
use shared::old_types::{EmoteObjectId, TicketObjectId};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::{Report, ReportStatus};

#[derive(Default)]
pub struct ReportsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl ReportsMutation {
	#[graphql(guard = "PermissionGuard::one(TicketPermission::Create)")]
	async fn create_report<'ctx>(&self, ctx: &Context<'ctx>, data: CreateReportInput) -> Result<Report, ApiError> {
		if data.target_kind != 2 {
			return Err(ApiError::NOT_IMPLEMENTED);
		}

		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_sesion = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let mut session = global.mongo().start_session(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let ticket = database::Ticket {
			title: data.subject,
			data: database::TicketData::EmoteReport {
				emote_id: data.target_id.id(),
			},
			..Default::default()
		};

		database::Ticket::collection(global.db())
			.insert_one_with_session(&ticket, None, &mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert ticket");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let message = database::TicketMessage {
			ticket_id: ticket.id,
			user_id: auth_sesion.user_id(),
			content: data.body,
			..Default::default()
		};

		database::TicketMessage::collection(global.db())
			.insert_one_with_session(&message, None, &mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert ticket message");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let member = database::TicketMember {
			ticket_id: ticket.id,
			user_id: auth_sesion.user_id(),
			notifications: true,
			last_read: Some(message.id),
			..Default::default()
		};

		database::TicketMember::collection(global.db())
			.insert_one_with_session(&member, None, &mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert ticket member");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Report::from_db(ticket, vec![member], vec![message]).ok_or(ApiError::INTERNAL_SERVER_ERROR)
	}

	#[graphql(guard = "PermissionGuard::one(TicketPermission::Edit)")]
	async fn edit_report<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		report_id: TicketObjectId,
		data: EditReportInput,
	) -> Result<Report, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_sesion = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let mut session = global.mongo().start_session(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let mut update = doc! {};

		if let Some(priority) = data.priority {
			let priority = (priority > 3).then(|| 3).unwrap_or(priority);
			update.insert("priority", priority);
		}

		if let Some(status) = data.status {
			update.insert("status", TicketStatus::from(status) as u32);
		}

		let ticket = database::Ticket::collection(global.db())
			.find_one_and_update_with_session(
				doc! { "_id": report_id.id() },
				doc! { "$set": update },
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.build(),
				&mut session,
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update ticket");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::NOT_FOUND)?;

		if let Some(assignee) = data.assignee {
			let mut chars = assignee.chars();
			match (chars.next(), UserId::from_str(chars.as_str())) {
				(Some('+'), Ok(user_id)) => {
					let member = TicketMember {
						ticket_id: ticket.id,
						user_id,
						kind: TicketMemberKind::Staff,
						notifications: true,
						..Default::default()
					};
					database::TicketMember::collection(global.db())
						.insert_one_with_session(member, None, &mut session)
						.await
						.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
				}
				(Some('-'), Ok(user_id)) => {
					database::TicketMember::collection(global.db())
						.delete_one_with_session(doc! { "ticket_id": ticket.id, "user_id": user_id }, None, &mut session)
						.await
						.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
				}
				_ => return Err(ApiError::BAD_REQUEST),
			}
		}

		if let Some(note) = data.note {
			let message = database::TicketMessage {
				ticket_id: ticket.id,
				user_id: auth_sesion.user_id(),
				content: note.content.unwrap_or_default(),
				..Default::default()
			};

			database::TicketMessage::collection(global.db())
				.insert_one_with_session(&message, None, &mut session)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to insert ticket message");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		}

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let members = global
			.ticket_members_by_ticket_id_loader()
			.load(ticket.id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		let messages = global
			.ticket_messages_by_ticket_id_loader()
			.load(ticket.id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Report::from_db(ticket, members, messages).ok_or(ApiError::INTERNAL_SERVER_ERROR)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateReportInput {
	target_kind: u32,
	target_id: EmoteObjectId,
	subject: String,
	body: String,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditReportInput {
	priority: Option<u32>,
	status: Option<ReportStatus>,
	assignee: Option<String>,
	note: Option<EditReportNoteInput>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditReportNoteInput {
	timestamp: Option<String>,
	content: Option<String>,
	internal: Option<bool>,
	reply: Option<String>,
}
