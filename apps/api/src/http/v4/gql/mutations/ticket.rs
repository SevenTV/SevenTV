use std::sync::Arc;

use async_graphql::Context;
use shared::database::ticket::{TicketId, TicketMessageId};
use shared::database::Id;
use shared::event::{InternalEvent, InternalEventData, InternalEventTicketData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::{Ticket, TicketTargetType};
use crate::transactions::{transaction, TransactionError};

#[derive(Default)]
pub struct TicketMutation;

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct TicketTargetInput {
	kind: TicketTargetType,
	id: Id<()>,
}

impl From<TicketTargetInput> for shared::database::ticket::TicketTarget {
	fn from(value: TicketTargetInput) -> Self {
		match value.kind {
			TicketTargetType::User => shared::database::ticket::TicketTarget::User(value.id.cast()),
			TicketTargetType::Emote => shared::database::ticket::TicketTarget::Emote(value.id.cast()),
			TicketTargetType::EmoteSet => shared::database::ticket::TicketTarget::EmoteSet(value.id.cast()),
		}
	}
}

#[async_graphql::Object]
impl TicketMutation {
	async fn create_abuse_ticket(
		&self,
		ctx: &Context<'_>,
		target: TicketTargetInput,
		#[graphql(validator(min_length = 1, max_length = 100))] title: String,
		#[graphql(validator(min_length = 1, max_length = 1000))] content: Option<String>,
	) -> Result<Ticket, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let country_code = global
			.geoip()
			.and_then(|g| g.lookup(session.ip()))
			.and_then(|c| c.iso_code)
			.map(|c| c.to_string());

		let res = transaction(global, |mut tx| async move {
			let ticket_id = TicketId::new();

			let message_id = if let Some(c) = content {
				let id = TicketMessageId::new();

				let message = shared::database::ticket::TicketMessage {
					id,
					ticket_id,
					user_id: authed_user.id,
					content: c,
					files: vec![],
					search_updated_at: None,
					updated_at: chrono::Utc::now(),
				};

				tx.insert_one(message, None).await?;

				Some(id)
			} else {
				None
			};

			let member = shared::database::ticket::TicketMember {
				user_id: authed_user.id,
				kind: shared::database::ticket::TicketMemberKind::Member,
				notifications: true,
				last_read: message_id,
			};

			let ticket = shared::database::ticket::Ticket {
				id: ticket_id,
				priority: shared::database::ticket::TicketPriority::Medium,
				members: vec![member],
				title,
				tags: vec![],
				country_code,
				kind: shared::database::ticket::TicketKind::Abuse,
				targets: vec![target.into()],
				author_id: authed_user.id,
				open: true,
				locked: false,
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			};

			tx.insert_one::<shared::database::ticket::Ticket>(&ticket, None).await?;

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::Ticket {
					after: ticket.clone(),
					data: InternalEventTicketData::Create,
				},
				timestamp: chrono::Utc::now(),
			})?;

			Ok(ticket)
		})
		.await;

		match res {
			Ok(ticket) => Ok(Ticket::from(ticket)),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}
}
