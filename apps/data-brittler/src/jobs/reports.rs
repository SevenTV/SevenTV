use std::sync::Arc;

use fnv::FnvHashSet;
use mongodb::options::InsertManyOptions;
use shared::database::{self, Collection, Ticket, TicketId, TicketKind, TicketMember, TicketMemberId, TicketMessage, TicketMessageId, TicketPriority, UserId};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct ReportsJob {
	global: Arc<Global>,
	all_members: FnvHashSet<(TicketId, UserId)>,
	tickets: Vec<Ticket>,
	ticket_members: Vec<TicketMember>,
	ticket_messages: Vec<TicketMessage>,
}

impl Job for ReportsJob {
	type T = types::Report;

	const NAME: &'static str = "transfer_reports";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping tickets and ticket_members collections");
			Ticket::collection(global.target_db()).drop(None).await?;
			TicketMember::collection(global.target_db()).drop(None).await?;
		}

		Ok(Self {
			global,
			all_members: FnvHashSet::default(),
			tickets: vec![],
			ticket_members: vec![],
			ticket_messages: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("reports")
	}

	async fn process(&mut self, report: Self::T) -> ProcessOutcome {
		// Only emote reports because reporting users was never implemented

		let ticket_id = report.id.into();

		self.tickets.push(Ticket {
			id: ticket_id,
			kind: TicketKind::EmoteReport,
			status: report.status.into(),
			priority: TicketPriority::Low,
			title: report.subject,
			tags: vec![],
		});

		let message_id = TicketMessageId::with_timestamp(report.id.timestamp().to_chrono());

		self.ticket_messages.push(TicketMessage {
			id: message_id,
			ticket_id,
			user_id: report.actor_id.into(),
			content: report.body,
			files: vec![],
		});

		let op = report.actor_id.into();
		self.ticket_members.push(TicketMember {
			id: TicketMemberId::new(),
			ticket_id,
			user_id: op,
			kind: database::TicketMemberKind::Op,
			notifications: true,
			last_read: Some(message_id),
		});
		self.all_members.insert((ticket_id, op));

		for assignee in report.assignee_ids {
			let assignee = assignee.into();
			if self.all_members.insert((ticket_id, assignee)) {
				self.ticket_members.push(TicketMember {
					id: TicketMemberId::new(),
					ticket_id,
					user_id: assignee,
					kind: database::TicketMemberKind::Staff,
					notifications: true,
					last_read: Some(message_id),
				});
			}
		}

		ProcessOutcome::default()
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing reports job");

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let tickets = Ticket::collection(self.global.target_db());
		let ticket_members = TicketMember::collection(self.global.target_db());
		let ticket_messages = TicketMessage::collection(self.global.target_db());

		let res = tokio::join!(
			tickets.insert_many(&self.tickets, insert_options.clone()),
			ticket_members.insert_many(&self.ticket_members, insert_options.clone()),
			ticket_messages.insert_many(&self.ticket_messages, insert_options.clone()),
		);
		let res = vec![res.0, res.1]
			.into_iter()
			.zip(vec![self.tickets.len(), self.ticket_members.len(), self.ticket_messages.len()]);

		for (res, len) in res {
			match res {
				Ok(res) => {
					outcome.inserted_rows += res.inserted_ids.len() as u64;
					if res.inserted_ids.len() != len {
						outcome.errors.push(error::Error::InsertMany);
					}
				}
				Err(e) => outcome.errors.push(e.into()),
			}
		}

		outcome
	}
}
