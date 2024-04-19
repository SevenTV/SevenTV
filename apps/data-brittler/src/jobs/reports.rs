use std::sync::Arc;

use fnv::FnvHashSet;
use mongodb::bson::oid::ObjectId;
use mongodb::options::InsertManyOptions;
use shared::database::{self, Collection, Ticket, TicketKind, TicketMember, TicketMemberId, TicketPriority};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct ReportsJob {
	global: Arc<Global>,
	all_members: FnvHashSet<(ObjectId, ObjectId)>,
	tickets: Vec<Ticket>,
	ticket_members: Vec<TicketMember>,
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
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("reports")
	}

	async fn process(&mut self, report: Self::T) -> ProcessOutcome {
		// Only emote reports because reporting users was never implemented

		self.tickets.push(Ticket {
			id: report.id.into(),
			kind: TicketKind::EmoteReport,
			status: report.status.into(),
			priority: TicketPriority::Low,
			title: report.subject,
			tags: vec![],
			files: vec![],
		});

		let op = report.actor_id;
		self.ticket_members.push(TicketMember {
			id: TicketMemberId::new(),
			ticket_id: report.id.into(),
			user_id: op.into(),
			kind: database::TicketMemberKind::Op,
			notifications: true,
		});
		self.all_members.insert((report.id, op));

		for assignee in report.assignee_ids {
			if self.all_members.insert((report.id, assignee)) {
				self.ticket_members.push(TicketMember {
					id: TicketMemberId::new(),
					ticket_id: report.id.into(),
					user_id: assignee.into(),
					kind: database::TicketMemberKind::Staff,
					notifications: true,
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

		let res = tokio::join!(
			tickets.insert_many(&self.tickets, insert_options.clone()),
			ticket_members.insert_many(&self.ticket_members, insert_options.clone()),
		);
		let res = vec![res.0, res.1]
			.into_iter()
			.zip(vec![self.tickets.len(), self.ticket_members.len()]);

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
