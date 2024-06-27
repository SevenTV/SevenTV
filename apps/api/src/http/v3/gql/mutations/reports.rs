use std::str::FromStr;
use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use mongodb::bson::{doc, to_bson};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::ticket::{Ticket, TicketKind, TicketMember, TicketMemberKind, TicketMessage, TicketPriority, TicketTarget};
use shared::database::role::permissions::TicketPermission;
use shared::database::user::UserId;
use shared::database::Collection;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::report::{Report, ReportStatus};

#[derive(Default)]
pub struct ReportsMutation;


#[repr(u8)]
enum ObjectKind {
	User = 1,
	Emote = 2,
	EmoteSet = 3,
}

impl ObjectKind {
	fn from_u32(kind: u32) -> Option<Self> {
		Some(match kind {
			1 => Self::User,
			2 => Self::Emote,
			3 => Self::EmoteSet,
			_ => return None,
		})
	}
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl ReportsMutation {
	#[graphql(guard = "PermissionGuard::one(TicketPermission::Create)")]
	async fn create_report<'ctx>(&self, ctx: &Context<'ctx>, data: CreateReportInput) -> Result<Report, ApiError> {
		let kind = ObjectKind::from_u32(data.target_kind).ok_or(ApiError::BAD_REQUEST)?;

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

		let ticket = Ticket {
			id: Default::default(),
			title: data.subject,
			priority: TicketPriority::Medium,
			kind: TicketKind::Abuse,
			author_id: auth_sesion.user_id(),
			locked: false,
			open: true,
			members: vec![TicketMember {
				kind: TicketMemberKind::Member,
				last_read: None,
				notifications: true,
				user_id: auth_sesion.user_id(),
			}],
			tags: vec![],
			targets: vec![match kind {
				ObjectKind::User => TicketTarget::User(data.target_id.id()),
				ObjectKind::Emote => TicketTarget::Emote(data.target_id.id()),
				ObjectKind::EmoteSet => TicketTarget::EmoteSet(data.target_id.id()),
			}],
			country_code: None,
		};

		Ticket::collection(global.db())
			.insert_one_with_session(&ticket, None, &mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert ticket");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let message = TicketMessage {
			ticket_id: ticket.id,
			user_id: auth_sesion.user_id(),
			content: data.body,
			..Default::default()
		};

		TicketMessage::collection(global.db())
			.insert_one_with_session(&message, None, &mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert ticket message");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Report::from_db(ticket, vec![message]).ok_or(ApiError::INTERNAL_SERVER_ERROR)
	}

	#[graphql(guard = "PermissionGuard::one(TicketPermission::ManageAbuse)")]
	async fn edit_report<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		report_id: GqlObjectId,
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
			let priority = match priority {
				0 => TicketPriority::Low,
				1 => TicketPriority::Medium,
				2 => TicketPriority::High,
				_ => TicketPriority::Urgent,
			};
			update.insert("priority", to_bson(&priority).unwrap());
		}

		if let Some(status) = data.status {
			match status {
				ReportStatus::Open => {
					update.insert("open", true);
					update.insert("locked", false);
				},
				ReportStatus::Closed => {
					update.insert("open", false);
					update.insert("locked", true);
				},
				_ => {}
			};
		}

		let mut ticket = Ticket::collection(global.db())
			.find_one_and_update_with_session(
				doc! { "_id": report_id.0 },
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
					if !ticket.members.iter().any(|m| m.user_id == user_id) {
						let member = TicketMember {
							kind: TicketMemberKind::Member,
							last_read: None,
							notifications: true,
							user_id,
						};

						ticket = Ticket::collection(global.db())
							.find_one_and_update_with_session(
								doc! { "_id": ticket.id },
								doc! { "$push": { "members": to_bson(&member).unwrap() } },
								None,
								&mut session,
							)
							.await
							.map_err(|e| {
								tracing::error!(error = %e, "failed to add ticket member");
								ApiError::INTERNAL_SERVER_ERROR
							})?
							.ok_or(ApiError::NOT_FOUND)?;
					}
				}
				(Some('-'), Ok(user_id)) => {
					ticket = Ticket::collection(global.db())
						.find_one_and_update_with_session(
							doc! { "_id": ticket.id },
							doc! { "$pull": { "members": { "user_id": user_id } } },
							None,
							&mut session,
						)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to remove ticket member");
							ApiError::INTERNAL_SERVER_ERROR
						})?
						.ok_or(ApiError::NOT_FOUND)?;
				}
				_ => return Err(ApiError::BAD_REQUEST),
			}
		}

		if let Some(note) = data.note {
			let message = TicketMessage {
				ticket_id: ticket.id,
				user_id: auth_sesion.user_id(),
				content: note.content.unwrap_or_default(),
				..Default::default()
			};

			TicketMessage::collection(global.db())
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

		let messages = global
			.ticket_messages_by_ticket_id_loader()
			.load(ticket.id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

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
