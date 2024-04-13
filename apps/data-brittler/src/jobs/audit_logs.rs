use std::{mem, sync::Arc};

use clickhouse::Row;
use shared::{
	database::{self, Table},
	types::old::EmoteFlagsModel,
};

use crate::{
	error,
	global::Global,
	types::{self, AuditLogChange, AuditLogChangeArray, AuditLogKind},
};

use super::{Job, ProcessOutcome};

const BATCH_SIZE: u32 = 10_000;

pub struct AuditLogsJob {
	global: Arc<Global>,
	i: u32,
	emote_activity_writer: clickhouse::insert::Insert<database::EmoteActivity>,
	emote_set_activity_writer: clickhouse::insert::Insert<database::EmoteSetActivity>,
	user_activity_writer: clickhouse::insert::Insert<database::UserActivity>,
	ticket_activity_writer: clickhouse::insert::Insert<database::TicketActivity>,
}

async fn renew_writer<T: Row>(
	global: &Arc<Global>,
	old_writer: &mut clickhouse::insert::Insert<T>,
	table: &str,
) -> Option<error::Error> {
	mem::replace(
		old_writer,
		global.clickhouse().insert(table).map_err(Into::<error::Error>::into).ok()?,
	)
	.end()
	.await
	.err()
	.map(Into::into)
}

impl Job for AuditLogsJob {
	const NAME: &'static str = "transfer_audit_logs";

	type T = types::AuditLog;

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!(
				"truncating emote_activities, emote_set_activities, user_activities and ticket_activities tables"
			);

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
			i: 0,
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
			AuditLogKind::CreateEmote
			| AuditLogKind::ProcessEmote
			| AuditLogKind::UpdateEmote
			| AuditLogKind::MergeEmote
			| AuditLogKind::DeleteEmote => {
				let kind = match audit_log.kind {
					AuditLogKind::CreateEmote => database::EmoteActivityKind::Upload,
					AuditLogKind::ProcessEmote => database::EmoteActivityKind::Process,
					AuditLogKind::UpdateEmote => database::EmoteActivityKind::Edit,
					AuditLogKind::MergeEmote => database::EmoteActivityKind::Merge,
					AuditLogKind::DeleteEmote => database::EmoteActivityKind::Delete,
					_ => unreachable!(),
				};

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.filter_map(|c| {
						match c {
							AuditLogChange::Name(names) => Some(database::EmoteActivityData::ChangeName {
								old: names.old,
								new: names.new,
							}),
							AuditLogChange::EmoteVersions(AuditLogChangeArray { updated, .. }) => {
								let old = updated.iter().map(|u| &u.old).fold(
									database::EmoteSettingsChange::default(),
									|sum, c| database::EmoteSettingsChange {
										public_listed: sum.public_listed.or(c.listed),
										approved_personal: sum.approved_personal.or(c.allow_personal),
										..Default::default()
									},
								);
								let new = updated.iter().map(|u| &u.new).fold(
									database::EmoteSettingsChange::default(),
									|sum, c| database::EmoteSettingsChange {
										public_listed: sum.public_listed.or(c.listed),
										approved_personal: sum.approved_personal.or(c.allow_personal),
										..Default::default()
									},
								);
								Some(database::EmoteActivityData::ChangeSettings { old, new })
							}
							AuditLogChange::NewEmoteId(_) => {
								// TODO
								Some(database::EmoteActivityData::Merge {})
							}
							AuditLogChange::Tags(tags) => Some(database::EmoteActivityData::ChangeTags {
								added: tags.new,
								removed: tags.old,
							}),
							AuditLogChange::Flags(flags) => {
								let mut old = database::EmoteSettingsChange::default();
								if flags.old.contains(EmoteFlagsModel::Sexual) {
									old.nsfw = Some(true);
								}
								if flags.old.contains(EmoteFlagsModel::Private) {
									old.private = Some(true);
								}
								if flags.old.contains(EmoteFlagsModel::ZeroWidth) {
									old.default_zero_width = Some(true);
								}

								let mut new = database::EmoteSettingsChange::default();
								if flags.new.contains(EmoteFlagsModel::Sexual) {
									new.nsfw = Some(true);
								}
								if flags.new.contains(EmoteFlagsModel::Private) {
									new.private = Some(true);
								}
								if flags.new.contains(EmoteFlagsModel::ZeroWidth) {
									new.default_zero_width = Some(true);
								}

								Some(database::EmoteActivityData::ChangeSettings { old, new })
							}
							AuditLogChange::Owner(owner) => {
								if let (Some(old), Some(new)) = (owner.old.into_inner(), owner.new.into_inner()) {
									Some(database::EmoteActivityData::ChangeOwner {
										old: old.into_ulid(),
										new: new.into_ulid(),
									})
								} else {
									// TODO: do something here?
									None
								}
							}
							_ => unimplemented!(),
						}
					})
					.map(Some)
					.collect();

				if changes.is_empty() {
					changes.push(None);
				}

				for data in changes {
					let activity = database::EmoteActivity {
						emote_id: audit_log.target_id.into_uuid(),
						actor_id: Some(audit_log.actor_id.into_uuid()),
						kind,
						data,
						timestamp,
					};

					match self.emote_activity_writer.write(&activity).await {
						Ok(_) => outcome.inserted_rows += 1,
						Err(e) => outcome.errors.push(e.into()),
					}
				}
			}
			AuditLogKind::CreateEmoteSet | AuditLogKind::UpdateEmoteSet | AuditLogKind::DeleteEmoteSet => {
				let kind = match audit_log.kind {
					AuditLogKind::CreateEmoteSet => database::EmoteSetActivityKind::Create,
					AuditLogKind::UpdateEmoteSet => database::EmoteSetActivityKind::Edit,
					AuditLogKind::DeleteEmoteSet => database::EmoteSetActivityKind::Delete,
					_ => unreachable!(),
				};

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.map(|c| match c {
						AuditLogChange::Name(names) => database::EmoteSetActivityData::ChangeName {
							old: names.old,
							new: names.new,
						},
						AuditLogChange::EmoteSetCapacity(c) => {
							let old = database::EmoteSetSettingsChange {
								capacity: Some(c.old as u32),
								..Default::default()
							};
							let new = database::EmoteSetSettingsChange {
								capacity: Some(c.new as u32),
								..Default::default()
							};
							database::EmoteSetActivityData::ChangeSettings { old, new }
						}
						AuditLogChange::EmoteSetEmotes(emotes) => {
							let added = emotes
								.added
								.into_iter()
								.filter_map(|e| e.id)
								.map(|id| id.into_ulid())
								.collect();
							let removed = emotes
								.removed
								.into_iter()
								.filter_map(|e| e.id)
								.map(|id| id.into_ulid())
								.collect();
							database::EmoteSetActivityData::ChangeEmotes { added, removed }
						}
						_ => unimplemented!(),
					})
					.map(Some)
					.collect();

				if changes.is_empty() {
					changes.push(None);
				}

				for data in changes {
					let activity = database::EmoteSetActivity {
						emote_set_id: audit_log.target_id.into_uuid(),
						actor_id: Some(audit_log.actor_id.into_uuid()),
						kind,
						data,
						timestamp,
					};

					match self.emote_set_activity_writer.write(&activity).await {
						Ok(_) => outcome.inserted_rows += 1,
						Err(e) => outcome.errors.push(e.into()),
					}
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

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.map(|c| match c {
						AuditLogChange::UserEditors(editors) => database::UserActivityData::ChangeEditors {
							added: editors
								.added
								.into_iter()
								.filter_map(|e| e.id)
								.map(|id| id.into_ulid())
								.collect(),
							removed: editors
								.removed
								.into_iter()
								.filter_map(|e| e.id)
								.map(|id| id.into_ulid())
								.collect(),
						},
						AuditLogChange::UserRoles(roles) => database::UserActivityData::ChangeRoles {
							added: roles.added.into_iter().filter_map(|e| e).map(|id| id.into_ulid()).collect(),
							removed: roles.removed.into_iter().filter_map(|e| e).map(|id| id.into_ulid()).collect(),
						},
						_ => unimplemented!(),
					})
					.map(Some)
					.collect();

				if changes.is_empty() {
					changes.push(None);
				}

				for data in changes {
					let activity = database::UserActivity {
						user_id: audit_log.target_id.into_uuid(),
						actor_id: Some(audit_log.actor_id.into_uuid()),
						kind,
						data,
						timestamp,
					};

					match self.user_activity_writer.write(&activity).await {
						Ok(_) => outcome.inserted_rows += 1,
						Err(e) => outcome.errors.push(e.into()),
					}
				}
			}
			AuditLogKind::CreateReport | AuditLogKind::UpdateReport => {
				let kind = match audit_log.kind {
					AuditLogKind::CreateReport => database::TicketActivityKind::Create,
					AuditLogKind::UpdateReport => database::TicketActivityKind::Edit,
					_ => unreachable!(),
				};

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.map(|c| match c {
						AuditLogChange::ReportStatus(status) => database::TicketActivityData::ChangeStatus {
							old: status.old.into(),
							new: status.new.into(),
						},
						AuditLogChange::ReportAssignees(assignees) => database::TicketActivityData::ChangeAssignees {
							added: assignees.added.into_iter().map(|id| id.into_ulid()).collect(),
							removed: assignees.removed.into_iter().map(|id| id.into_ulid()).collect(),
						},
						_ => unimplemented!(),
					})
					.map(Some)
					.collect();

				if changes.is_empty() {
					changes.push(None);
				}

				for data in changes {
					let activity = database::TicketActivity {
						ticket_id: audit_log.target_id.into_uuid(),
						actor_id: Some(audit_log.actor_id.into_uuid()),
						kind,
						data,
						timestamp,
					};

					match self.ticket_activity_writer.write(&activity).await {
						Ok(_) => outcome.inserted_rows += 1,
						Err(e) => {
							tracing::error!("{e}");
							outcome.errors.push(e.into());
						}
					}
				}
			}
			k => outcome.errors.push(error::Error::UnsupportedAuditLogKind(k)),
		}

		self.i += 1;
		if self.i > BATCH_SIZE {
			renew_writer(
				&self.global,
				&mut self.emote_activity_writer,
				database::EmoteActivity::TABLE_NAME,
			)
			.await;
			renew_writer(
				&self.global,
				&mut self.emote_set_activity_writer,
				database::EmoteSetActivity::TABLE_NAME,
			)
			.await;
			renew_writer(
				&self.global,
				&mut self.user_activity_writer,
				database::UserActivity::TABLE_NAME,
			)
			.await;
			renew_writer(
				&self.global,
				&mut self.ticket_activity_writer,
				database::TicketActivity::TABLE_NAME,
			)
			.await;
			self.i = 0;
		}

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
