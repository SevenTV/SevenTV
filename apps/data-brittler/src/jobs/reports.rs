use std::future::IntoFuture;
use std::sync::Arc;

use fnv::FnvHashSet;
use mongodb::options::InsertManyOptions;
use shared::database::ticket::{
	Ticket, TicketId, TicketKind, TicketMember, TicketMemberKind, TicketMessage, TicketMessageId, TicketPriority,
	TicketTarget,
};
use shared::database::user::UserId;
use shared::database::Collection;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::ReportStatus;
use crate::{error, types};

pub struct ReportsJob {
	global: Arc<Global>,
	all_members: FnvHashSet<(TicketId, UserId)>,
	tickets: Vec<Ticket>,
	ticket_messages: Vec<TicketMessage>,
}

impl Job for ReportsJob {
	type T = types::Report;

	const NAME: &'static str = "transfer_reports";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping tickets and ticket_messages collections");
			Ticket::collection(global.target_db()).drop().await?;
			let indexes = Ticket::indexes();
			if !indexes.is_empty() {
				Ticket::collection(global.target_db()).create_indexes(indexes).await?;
			}
			TicketMessage::collection(global.target_db()).drop().await?;
			let indexes = TicketMessage::indexes();
			if !indexes.is_empty() {
				TicketMessage::collection(global.target_db()).create_indexes(indexes).await?;
			}
		}

		Ok(Self {
			global,
			all_members: FnvHashSet::default(),
			tickets: vec![],
			ticket_messages: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("reports")
	}

	async fn process(&mut self, report: Self::T) -> ProcessOutcome {
		// Only emote reports because reporting users was never implemented

		let ticket_id = report.id.into();

		let message_id = TicketMessageId::with_timestamp(report.id.timestamp().to_chrono());

		self.ticket_messages.push(TicketMessage {
			id: message_id,
			ticket_id,
			user_id: report.actor_id.into(),
			content: report.body,
			files: vec![],
		});

		let mut members = vec![];

		let op = report.actor_id.into();
		members.push(TicketMember {
			user_id: op,
			kind: TicketMemberKind::Member,
			notifications: true,
			last_read: Some(message_id),
		});
		self.all_members.insert((ticket_id, op));

		for assignee in report.assignee_ids {
			let assignee = assignee.into();
			if self.all_members.insert((ticket_id, assignee)) {
				members.push(TicketMember {
					user_id: assignee,
					kind: TicketMemberKind::Assigned,
					notifications: true,
					last_read: Some(message_id),
				});
			}
		}

		self.tickets.push(Ticket {
			id: ticket_id,
			priority: TicketPriority::Medium,
			members,
			title: report.subject,
			tags: vec![],
			country_code: None,
			kind: TicketKind::Abuse,
			targets: vec![TicketTarget::Emote(report.target_id.into())],
			author_id: report.actor_id.into(),
			open: report.status == ReportStatus::Open,
			locked: false,
		});

		ProcessOutcome::default()
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing reports job");

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let tickets = Ticket::collection(self.global.target_db());
		let ticket_messages = TicketMessage::collection(self.global.target_db());

		let res = tokio::join!(
			tickets
				.insert_many(&self.tickets)
				.with_options(insert_options.clone())
				.into_future(),
			ticket_messages
				.insert_many(&self.ticket_messages)
				.with_options(insert_options.clone())
				.into_future(),
		);
		let res = vec![res.0, res.1]
			.into_iter()
			.zip(vec![self.tickets.len(), self.ticket_messages.len()]);

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
