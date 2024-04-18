use std::sync::Arc;

use fnv::FnvHashSet;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use shared::database::{self, Collection, Ticket, TicketKind, TicketMember, TicketPriority};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

// Only emote reports because reporting users was never implemented

pub struct ReportsJob {
	global: Arc<Global>,
	all_members: FnvHashSet<(ObjectId, ObjectId)>,
}

impl Job for ReportsJob {
	type T = types::Report;

	const NAME: &'static str = "transfer_reports";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating tickets and ticket_members collections");
			Ticket::collection(global.target_db()).delete_many(doc! {}, None).await?;
			TicketMember::collection(global.target_db())
				.delete_many(doc! {}, None)
				.await?;
		}

		Ok(Self {
			global,
			all_members: FnvHashSet::default(),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("reports")
	}

	async fn process(&mut self, report: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		// TODO: target id
		match Ticket::collection(self.global.target_db())
			.insert_one(
				Ticket {
					id: report.id,
					kind: TicketKind::EmoteReport,
					status: report.status.into(),
					priority: TicketPriority::Low,
					title: report.subject,
					tags: vec![],
					files: vec![],
				},
				None,
			)
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => {
				outcome.errors.push(e.into());
				return outcome;
			}
		}

		let op = report.actor_id;
		match TicketMember::collection(self.global.target_db())
			.insert_one(
				TicketMember {
					id: ObjectId::new(),
					ticket_id: report.id,
					user_id: op,
					kind: database::TicketMemberKind::Op,
					notifications: true,
				},
				None,
			)
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}
		self.all_members.insert((report.id, op));

		for assignee in report.assignee_ids {
			if self.all_members.insert((report.id, assignee)) {
				match TicketMember::collection(self.global.target_db())
					.insert_one(
						TicketMember {
							id: ObjectId::new(),
							ticket_id: report.id,
							user_id: assignee,
							kind: database::TicketMemberKind::Staff,
							notifications: true,
						},
						None,
					)
					.await
				{
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => outcome.errors.push(e.into()),
				}
			}
		}

		outcome
	}
}
