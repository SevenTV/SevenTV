use std::str::FromStr;
use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use chrono::Utc;
use mongodb::bson::{doc, to_bson};
use mongodb::options::ReturnDocument;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogId, AuditLogTicketData};
use shared::database::role::permissions::TicketPermission;
use shared::database::ticket::{
	Ticket, TicketId, TicketKind, TicketMember, TicketMemberKind, TicketMessage, TicketMessageId, TicketPriority,
	TicketTarget,
};
use shared::database::user::UserId;
use shared::database::MongoCollection;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::report::{Report, ReportStatus};

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

		let mut session = global.mongo.start_session().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let ticket_id = TicketId::new();

		session.start_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let message = TicketMessage {
			id: TicketMessageId::new(),
			ticket_id,
			user_id: auth_sesion.user_id(),
			content: data.body,
			files: vec![],
			search_updated_at: None,
			updated_at: Utc::now(),
		};

		TicketMessage::collection(&global.db)
			.insert_one(&message)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert ticket message");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let member = TicketMember {
			user_id: auth_sesion.user_id(),
			kind: TicketMemberKind::Member,
			notifications: true,
			last_read: Some(message.id),
		};

		let ticket = Ticket {
			id: ticket_id,
			priority: TicketPriority::Medium,
			members: vec![member],
			title: data.subject,
			tags: vec![],
			country_code: None, // TODO
			kind: TicketKind::Abuse,
			targets: vec![TicketTarget::Emote(data.target_id.id())],
			author_id: auth_sesion.user_id(),
			open: true,
			locked: false,
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		};

		Ticket::collection(&global.db)
			.insert_one(&ticket)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert ticket");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		AuditLog::collection(&global.db)
			.insert_one(AuditLog {
				id: AuditLogId::new(),
				actor_id: Some(auth_sesion.user_id()),
				data: AuditLogData::Ticket {
					target_id: ticket.id,
					data: AuditLogTicketData::Create,
				},
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			})
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert audit log");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Report::from_db(ticket, vec![message]).ok_or(ApiError::INTERNAL_SERVER_ERROR)
	}

	#[graphql(guard = "PermissionGuard::all([TicketPermission::ManageAbuse, TicketPermission::ManageGeneric])")]
	async fn edit_report<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		report_id: GqlObjectId,
		data: EditReportInput,
	) -> Result<Report, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_sesion = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let ticket = global
			.ticket_by_id_loader
			.load(report_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		let mut session = global.mongo.start_session().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let mut update = doc! {};
		let mut push_update = doc! {};
		let mut pull_update = doc! {};

		if let Some(priority) = data.priority {
			update.insert(
				"priority",
				priority.clamp(TicketPriority::Low as i32, TicketPriority::Urgent as i32),
			);
		}

		if let Some(status) = data.status {
			let new = status == ReportStatus::Open || status == ReportStatus::Assigned;
			update.insert("open", new);

			if new != ticket.open {
				AuditLog::collection(&global.db)
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(auth_sesion.user_id()),
						data: AuditLogData::Ticket {
							target_id: report_id.id(),
							data: AuditLogTicketData::ChangeOpen {
								old: ticket.open,
								new: new,
							},
						},
						updated_at: chrono::Utc::now(),
						search_updated_at: None,
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}
		}

		if let Some(assignee) = data.assignee {
			let mut chars = assignee.chars();
			match (chars.next(), UserId::from_str(chars.as_str())) {
				(Some('+'), Ok(user_id)) => {
					let member = TicketMember {
						user_id,
						kind: TicketMemberKind::Assigned,
						notifications: true,
						last_read: None,
					};
					push_update.insert("members", to_bson(&member).expect("failed to convert member to bson"));

					AuditLog::collection(&global.db)
						.insert_one(AuditLog {
							id: AuditLogId::new(),
							actor_id: Some(auth_sesion.user_id()),
							data: AuditLogData::Ticket {
								target_id: report_id.id(),
								data: AuditLogTicketData::AddMember { member: user_id },
							},
							updated_at: chrono::Utc::now(),
							search_updated_at: None,
						})
						.session(&mut session)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to insert audit log");
							ApiError::INTERNAL_SERVER_ERROR
						})?;
				}
				(Some('-'), Ok(user_id)) => {
					pull_update.insert("members", doc! { "user_id": user_id });

					AuditLog::collection(&global.db)
						.insert_one(AuditLog {
							id: AuditLogId::new(),
							actor_id: Some(auth_sesion.user_id()),
							data: AuditLogData::Ticket {
								target_id: report_id.id(),
								data: AuditLogTicketData::RemoveMember { member: user_id },
							},
							updated_at: chrono::Utc::now(),
							search_updated_at: None,
						})
						.session(&mut session)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to insert audit log");
							ApiError::INTERNAL_SERVER_ERROR
						})?;
				}
				_ => return Err(ApiError::BAD_REQUEST),
			}
		}

		update.insert("updated_at", Some(bson::DateTime::from(chrono::Utc::now())));

		let ticket = Ticket::collection(&global.db)
			.find_one_and_update(
				doc! { "_id": report_id.0 },
				doc! { "$set": update, "$push": push_update, "$pull": pull_update },
			)
			.return_document(ReturnDocument::After)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update ticket");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::NOT_FOUND)?;

		if let Some(note) = data.note {
			let message = TicketMessage {
				id: TicketMessageId::new(),
				ticket_id: ticket.id,
				user_id: auth_sesion.user_id(),
				content: note.content.unwrap_or_default(),
				files: vec![],
				search_updated_at: None,
				updated_at: Utc::now(),
			};

			TicketMessage::collection(&global.db)
				.insert_one(&message)
				.session(&mut session)
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

		let messages = global
			.ticket_message_by_ticket_id_loader
			.load(ticket.id)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Report::from_db(ticket, messages).ok_or(ApiError::INTERNAL_SERVER_ERROR)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateReportInput {
	target_kind: u32,
	target_id: GqlObjectId,
	subject: String,
	body: String,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditReportInput {
	priority: Option<i32>,
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
