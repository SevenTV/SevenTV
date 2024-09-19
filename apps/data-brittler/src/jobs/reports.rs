use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::ticket::{
	Ticket, TicketId, TicketKind, TicketMember, TicketMemberKind, TicketMessage, TicketMessageId, TicketPriority,
	TicketTarget,
};
use shared::database::user::UserId;

use super::{JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::types;
use crate::types::ReportStatus;

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub tickets: &'a mut HashMap<TicketId, Ticket>,
	pub ticket_messages: &'a mut Vec<TicketMessage>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("reports");

	let RunInput {
		global,
		tickets,
		ticket_messages,
	} = input;

	let mut cursor = global
		.main_source_db
		.collection::<types::Report>("reports")
		.find(bson::doc! {})
		.await
		.context("query")?;

	let mut all_members = HashSet::new();

	while let Some(report) = cursor.next().await {
		match report {
			Ok(report) => {
				outcome += process(ProcessInput {
					all_members: &mut all_members,
					tickets,
					ticket_messages,
					report,
				});
				outcome.processed_documents += 1;
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	all_members: &'a mut HashSet<(TicketId, UserId)>,
	tickets: &'a mut HashMap<TicketId, Ticket>,
	ticket_messages: &'a mut Vec<TicketMessage>,
	report: types::Report,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let ProcessInput {
		all_members,
		tickets,
		ticket_messages,
		report,
	} = input;

	// Only emote reports because reporting users was never implemented
	let ticket_id = report.id.into();

	let message_id = TicketMessageId::with_timestamp(report.id.timestamp().to_chrono());

	ticket_messages.push(TicketMessage {
		id: message_id,
		ticket_id,
		user_id: report.actor_id.into(),
		content: report.body,
		files: vec![],
		search_updated_at: None,
		updated_at: chrono::Utc::now(),
	});

	let mut members = vec![];

	let op = report.actor_id.into();
	members.push(TicketMember {
		user_id: op,
		kind: TicketMemberKind::Member,
		notifications: true,
		last_read: Some(message_id),
	});
	all_members.insert((ticket_id, op));

	for assignee in report.assignee_ids {
		let assignee = assignee.into();
		if all_members.insert((ticket_id, assignee)) {
			members.push(TicketMember {
				user_id: assignee,
				kind: TicketMemberKind::Assigned,
				notifications: true,
				last_read: Some(message_id),
			});
		}
	}

	tickets.insert(
		ticket_id,
		Ticket {
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
			search_updated_at: None,
			updated_at: chrono::Utc::now(),
		},
	);

	ProcessOutcome::default()
}
