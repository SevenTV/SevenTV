use std::pin::Pin;
use std::sync::Arc;

use fnv::FnvHashSet;
use postgres_types::Type;
use shared::database;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use super::{Job, ProcessOutcome};
use crate::database::{ticket_kind_type, ticket_member_kind_type, ticket_priority_type, ticket_status_type};
use crate::global::Global;
use crate::types;

// Only emote reports because reporting users was never implemented

pub struct ReportsJob {
	global: Arc<Global>,
	tickets_writer: Pin<Box<BinaryCopyInWriter>>,
	ticket_members_writer: Pin<Box<BinaryCopyInWriter>>,
	all_members: FnvHashSet<(ulid::Ulid, ulid::Ulid)>,
}

impl Job for ReportsJob {
	type T = types::Report;

	const NAME: &'static str = "transfer_reports";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating tickets and ticket_members tables");
			scuffle_utils::database::query("TRUNCATE tickets, ticket_members")
				.build()
				.execute(global.db())
				.await?;
		}

		let tickets_client = global.db().get().await?;
		let tickets_writer = BinaryCopyInWriter::new(
			tickets_client
				.copy_in(
					"COPY tickets (id, kind, status, priority, title, data, updated_at) FROM STDIN WITH (FORMAT BINARY)",
				)
				.await?,
			&[
				Type::UUID,
				ticket_kind_type(&global).await?,
				ticket_status_type(&global).await?,
				ticket_priority_type(&global).await?,
				Type::TEXT,
				Type::JSONB,
				Type::TIMESTAMPTZ,
			],
		);

		let ticket_members_client = global.db().get().await?;
		let ticket_members_writer = BinaryCopyInWriter::new(
			ticket_members_client
				.copy_in("COPY ticket_members (ticket_id, user_id, kind) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, Type::UUID, ticket_member_kind_type(&global).await?],
		);

		Ok(Self {
			global,
			tickets_writer: Box::pin(tickets_writer),
			ticket_members_writer: Box::pin(ticket_members_writer),
			all_members: FnvHashSet::default(),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("reports")
	}

	async fn process(&mut self, report: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = report.id.into_ulid();

		// TODO: data
		let data = database::TicketData {};

		match self
			.tickets_writer
			.as_mut()
			.write(&[
				&id,
				&database::TicketKind::EmoteReport,
				&Into::<database::TicketStatus>::into(report.status),
				&database::TicketPriority::Low,
				&report.subject,
				&postgres_types::Json(data),
				&report.last_updated_at.into_chrono(),
			])
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => {
				outcome.errors.push(e.into());
				return outcome;
			}
		}

		let op = report.actor_id.into_ulid();
		match self
			.ticket_members_writer
			.as_mut()
			.write(&[&id, &op, &database::TicketMemberKind::Op])
			.await
		{
			Ok(_) => outcome.inserted_rows += 1,
			Err(e) => outcome.errors.push(e.into()),
		}
		self.all_members.insert((id, op));

		for assignee in report.assignee_ids {
			let assignee = assignee.into_ulid();

			if self.all_members.insert((id, assignee)) {
				match self
					.ticket_members_writer
					.as_mut()
					.write(&[&id, &assignee, &database::TicketMemberKind::Staff])
					.await
				{
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => outcome.errors.push(e.into()),
				}
			}
		}

		outcome
	}

	async fn finish(mut self) -> anyhow::Result<()> {
		self.tickets_writer.as_mut().finish().await?;
		self.ticket_members_writer.as_mut().finish().await?;
		Ok(())
	}
}
