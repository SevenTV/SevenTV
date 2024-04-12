use std::sync::Arc;

use shared::database::{self, Table};

use crate::{
	error,
	global::Global,
	types::{self, AuditLogKind},
};

use super::{Job, ProcessOutcome};

pub struct AuditLogsJob {
	global: Arc<Global>,
	// i: u32,
	emote_activity_writer: clickhouse::insert::Insert<database::EmoteActivity>,
	emote_set_activity_writer: clickhouse::insert::Insert<database::EmoteSetActivity>,
	user_activity_writer: clickhouse::insert::Insert<database::UserActivity>,
	ticket_activity_writer: clickhouse::insert::Insert<database::TicketActivity>,
}

impl Job for AuditLogsJob {
	const NAME: &'static str = "transfer_audit_logs";

	type T = types::AuditLog;

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating emote_activities, emote_set_activities, user_activities and ticket_activities tables");

			let conn = global.clickhouse();
			conn.query("TRUNCATE TABLE emote_activities").execute().await?;
			conn.query("TRUNCATE TABLE emote_set_activities").execute().await?;
			conn.query("TRUNCATE TABLE user_activities").execute().await?;
			conn.query("TRUNCATE TABLE ticket_activities").execute().await?;
		}

		let emote_activity_writer = global.clickhouse().insert(database::EmoteActivity::TABLE_NAME)?;
		let emote_set_activity_writer = global.clickhouse().insert(database::EmoteSetActivity::TABLE_NAME)?;
		let user_activity_writer = global.clickhouse().insert(database::UserActivity::TABLE_NAME)?;
		let ticket_activity_writer = global.clickhouse().insert(database::TicketActivity::TABLE_NAME)?;

		Ok(Self {
			global,
			// i: 0,
			emote_activity_writer,
			emote_set_activity_writer,
			user_activity_writer,
			ticket_activity_writer,
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("audit_logs")
	}

	async fn process(&mut self, audit_log: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let timestamp = match time::OffsetDateTime::from_unix_timestamp(audit_log.id.timestamp() as i64) {
			Ok(ts) => ts,
			Err(e) => {
				outcome.errors.push(e.into());
				return outcome;
			}
		};

		match audit_log.kind {
			AuditLogKind::CreateEmote | AuditLogKind::ProcessEmote | AuditLogKind::UpdateEmote | AuditLogKind::MergeEmote | AuditLogKind::DeleteEmote | AuditLogKind::UndoDeleteEmote => {
				let kind = match audit_log.kind {
					AuditLogKind::CreateEmote => database::EmoteActivityKind::Upload,
					AuditLogKind::ProcessEmote => database::EmoteActivityKind::Process,
					AuditLogKind::UpdateEmote => database::EmoteActivityKind::Edit,
					AuditLogKind::MergeEmote => database::EmoteActivityKind::Merge,
					AuditLogKind::DeleteEmote => database::EmoteActivityKind::Delete,
					AuditLogKind::UndoDeleteEmote => database::EmoteActivityKind::UndoDelete,
					_ => unreachable!(),
				};

				let activity = database::EmoteActivity {
					emote_id: audit_log.target_id.into_uuid(),
					actor_id: Some(audit_log.actor_id.into_uuid()),
					kind,
					timestamp,
				};

				match self.emote_activity_writer.write(&activity).await {
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => outcome.errors.push(e.into()),
				}
			}
			AuditLogKind::CreateEmoteSet | AuditLogKind::UpdateEmoteSet | AuditLogKind::DeleteEmoteSet => {
				let kind = match audit_log.kind {
					AuditLogKind::CreateEmoteSet => database::EmoteSetActivityKind::Create,
					AuditLogKind::UpdateEmoteSet => database::EmoteSetActivityKind::Edit,
					AuditLogKind::DeleteEmoteSet => database::EmoteSetActivityKind::Delete,
					_ => unreachable!(),
				};

				let activity = database::EmoteSetActivity {
					emote_set_id: audit_log.target_id.into_uuid(),
					actor_id: Some(audit_log.actor_id.into_uuid()),
					kind,
					timestamp,
				};

				match self.emote_set_activity_writer.write(&activity).await {
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => outcome.errors.push(e.into()),
				}
			}
			AuditLogKind::CreateUser
			| AuditLogKind::EditUser
			| AuditLogKind::DeleteUser
			| AuditLogKind::BanUser
			| AuditLogKind::UnbanUser => {
				let kind = match audit_log.kind {
					AuditLogKind::CreateUser => database::UserActivityKind::Register,
					AuditLogKind::EditUser => database::UserActivityKind::Edit,
					AuditLogKind::DeleteUser => database::UserActivityKind::Delete,
					AuditLogKind::BanUser => database::UserActivityKind::Ban,
					AuditLogKind::UnbanUser => database::UserActivityKind::Unban,
					_ => unreachable!(),
				};

				let activity = database::UserActivity {
					user_id: audit_log.target_id.into_uuid(),
					actor_id: Some(audit_log.actor_id.into_uuid()),
					kind,
					timestamp,
				};

				match self.user_activity_writer.write(&activity).await {
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => outcome.errors.push(e.into()),
				}
			}
			AuditLogKind::CreateReport | AuditLogKind::UpdateReport => {
				let kind = match audit_log.kind {
					AuditLogKind::CreateReport => database::TicketActivityKind::Create,
					AuditLogKind::UpdateReport => database::TicketActivityKind::Edit,
					_ => unreachable!(),
				};

				let activity = database::TicketActivity {
					ticket_id: audit_log.target_id.into_uuid(),
					actor_id: Some(audit_log.actor_id.into_uuid()),
					kind,
					timestamp,
				};

				match self.ticket_activity_writer.write(&activity).await {
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => {
						tracing::error!("{e}");
						outcome.errors.push(e.into());
					},
				}
			}
			k => outcome.errors.push(error::Error::UnsupportedAuditLogKind(k)),
		}

		// self.i += 1;
		// if self.i > 1_000_000 {
		// 	if let Err(e) = self.emote_activity_writer.commit().await {
		// 		outcome.errors.push(e.into());
		// 	}
		// 	if let Err(e) = self.emote_set_activity_writer.commit().await {
		// 		outcome.errors.push(e.into());
		// 	}
		// 	if let Err(e) = self.user_activity_writer.commit().await {
		// 		outcome.errors.push(e.into());
		// 	}
		// 	if let Err(e) = self.ticket_activity_writer.commit().await {
		// 		tracing::error!("failed to commit: {e}");
		// 		outcome.errors.push(e.into());
		// 	}
		// 	self.i = 0;
		// }

		outcome
	}

	async fn finish(self) -> anyhow::Result<()> {
		self.emote_activity_writer.end().await?;
		self.emote_set_activity_writer.end().await?;
		self.user_activity_writer.end().await?;
		self.ticket_activity_writer.end().await?;

		Ok(())
	}
}
