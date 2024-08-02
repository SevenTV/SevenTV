use std::str::FromStr;
use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use chrono::Utc;
use hyper::StatusCode;
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::event::{EventTicketData, EventTicketMessageData};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::TicketPermission;
use shared::database::ticket::{
	Ticket, TicketId, TicketKind, TicketMember, TicketMemberKind, TicketMessage, TicketMessageId, TicketPriority,
	TicketTarget,
};
use shared::database::user::UserId;
use shared::event::{EventPayload, EventPayloadData};
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::report::{Report, ReportStatus};
use crate::transactions::{with_transaction, TransactionError};

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

		let res = with_transaction(global, |mut tx| async move {
			let ticket_id = TicketId::new();

			let message = TicketMessage {
				id: TicketMessageId::new(),
				ticket_id,
				user_id: auth_sesion.user_id(),
				content: data.body,
				files: vec![],
				search_updated_at: None,
				updated_at: Utc::now(),
			};

			tx.insert_one::<TicketMessage>(&message, None).await?;

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

			tx.insert_one::<Ticket>(&ticket, None).await?;

			tx.register_event(EventPayload {
				actor_id: Some(auth_sesion.user_id()),
				data: EventPayloadData::Ticket {
					after: ticket.clone(),
					data: EventTicketData::Create,
				},
				timestamp: chrono::Utc::now(),
			})?;

			Ok((ticket, message))
		})
		.await;

		match res {
			Ok((ticket, message)) => Report::from_db(ticket, vec![message]).ok_or(ApiError::INTERNAL_SERVER_ERROR),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
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

		let transaction_result = with_transaction(global, |mut tx| async move {
			let new_priority = data
				.priority
				.map(|p| TicketPriority::try_from(p))
				.transpose()
				.map_err(|_| ApiError::new_const(StatusCode::BAD_REQUEST, "invalid ticket priority"))
				.map_err(TransactionError::custom)?;

			let new_open = if let Some(status) = data.status {
				let new = status == ReportStatus::Open || status == ReportStatus::Assigned;

				(new != ticket.open).then_some(new)
			} else {
				None
			};

			let mut update: update::Update<_> = update::update! {
				#[query(set)]
				Ticket {
					#[query(optional)]
					open: new_open,
					#[query(serde, optional)]
					priority: new_priority.as_ref(),
					updated_at: chrono::Utc::now(),
				},
			}
			.into();

			let mut event_ticket_data = None;

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

						event_ticket_data = Some(EventTicketData::AddMember { member: user_id });

						update = update.extend_one(update::update! {
							#[query(push)]
							Ticket {
								#[query(serde)]
								members: member,
							},
						});
					}
					(Some('-'), Ok(user_id)) => {
						event_ticket_data = Some(EventTicketData::RemoveMember { member: user_id });

						update = update.extend_one(update::update! {
							#[query(pull)]
							Ticket {
								members: TicketMember {
									user_id,
								},
							},
						});
					}
					_ => return Err(TransactionError::custom(ApiError::BAD_REQUEST)),
				}
			}

			let ticket = tx
				.find_one_and_update(
					filter::filter! {
						Ticket {
							#[query(rename = "_id")]
							id: report_id.id(),
						}
					},
					update,
					FindOneAndUpdateOptions::builder()
						.return_document(ReturnDocument::After)
						.build(),
				)
				.await?
				.ok_or(ApiError::NOT_FOUND)
				.map_err(TransactionError::custom)?;

			if let Some(new_priority) = new_priority {
				tx.register_event(EventPayload {
					actor_id: Some(auth_sesion.user_id()),
					data: EventPayloadData::Ticket {
						after: ticket.clone(),
						data: EventTicketData::ChangePriority {
							old: ticket.priority.clone(),
							new: new_priority,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if let Some(new_open) = new_open {
				tx.register_event(EventPayload {
					actor_id: Some(auth_sesion.user_id()),
					data: EventPayloadData::Ticket {
						after: ticket.clone(),
						data: EventTicketData::ChangeOpen {
							old: ticket.open,
							new: new_open,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if let Some(event_ticket_data) = event_ticket_data {
				tx.register_event(EventPayload {
					actor_id: Some(auth_sesion.user_id()),
					data: EventPayloadData::Ticket {
						after: ticket.clone(),
						data: event_ticket_data,
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

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

				tx.insert_one::<TicketMessage>(&message, None).await?;

				tx.register_event(EventPayload {
					actor_id: Some(auth_sesion.user_id()),
					data: EventPayloadData::TicketMessage {
						after: message,
						data: EventTicketMessageData::Create,
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			Ok(ticket)
		})
		.await;

		match transaction_result {
			Ok(ticket) => {
				let messages = global
					.ticket_message_by_ticket_id_loader
					.load(ticket.id)
					.await
					.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
					.unwrap_or_default();

				Report::from_db(ticket, messages).ok_or(ApiError::INTERNAL_SERVER_ERROR)
			}
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
	}
}

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateReportInput {
	target_kind: u32,
	target_id: GqlObjectId,
	subject: String,
	body: String,
}

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "snake_case")]
pub struct EditReportInput {
	priority: Option<i32>,
	status: Option<ReportStatus>,
	assignee: Option<String>,
	note: Option<EditReportNoteInput>,
}

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "snake_case")]
pub struct EditReportNoteInput {
	timestamp: Option<String>,
	content: Option<String>,
	internal: Option<bool>,
	reply: Option<String>,
}
