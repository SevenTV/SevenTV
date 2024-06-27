use std::mem;
use std::sync::Arc;

use clickhouse::Row;
use shared::database;
use shared::database::emote::EmoteId;
use shared::database::emote_set::EmoteSetId;
use shared::database::ticket::TicketId;
use shared::database::user::UserId;
use shared::old_types::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, AuditLogChange, AuditLogChangeArray, AuditLogKind, ReportStatus};

const BATCH_SIZE: u32 = 100_000;

pub struct AuditLogsJob {
	global: Arc<Global>,
	i: u32,
	emote_activity_writer: clickhouse::insert::Insert<database::activity::EmoteActivity>,
	emote_set_activity_writer: clickhouse::insert::Insert<database::activity::EmoteSetActivity>,
	user_activity_writer: clickhouse::insert::Insert<database::activity::UserActivity>,
	ticket_activity_writer: clickhouse::insert::Insert<database::activity::TicketActivity>,
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
	type T = types::AuditLog;

	const NAME: &'static str = "transfer_audit_logs";

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

		let emote_activity_writer = global.clickhouse().insert("emote_activities")?;
		let emote_set_activity_writer = global.clickhouse().insert("emote_set_activities")?;
		let user_activity_writer = global.clickhouse().insert("user_activities")?;
		let ticket_activity_writer = global.clickhouse().insert("ticket_activities")?;

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
		self.global.source_db().collection("audit_logs")
	}

	async fn process(&mut self, audit_log: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let timestamp = match time::OffsetDateTime::from_unix_timestamp(audit_log.id.timestamp().to_chrono().timestamp()) {
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
					AuditLogKind::CreateEmote => database::activity::EmoteActivityKind::Upload,
					AuditLogKind::ProcessEmote => database::activity::EmoteActivityKind::Process,
					AuditLogKind::UpdateEmote => database::activity::EmoteActivityKind::Edit,
					AuditLogKind::MergeEmote => database::activity::EmoteActivityKind::Merge,
					AuditLogKind::DeleteEmote => database::activity::EmoteActivityKind::Delete,
					_ => unreachable!(),
				};

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.filter_map(|c| {
						match c {
							AuditLogChange::Name(names) => Some(database::activity::EmoteActivityData::ChangeName {
								old: names.old,
								new: names.new,
							}),
							AuditLogChange::EmoteVersions(AuditLogChangeArray { updated, .. }) => {
								let old = updated.iter().map(|u| &u.old).fold(
									database::activity::EmoteSettingsChange::default(),
									|sum, c| database::activity::EmoteSettingsChange {
										public_listed: sum.public_listed.or(c.listed),
										approved_personal: sum.approved_personal.or(c.allow_personal),
										..Default::default()
									},
								);
								let new = updated.iter().map(|u| &u.new).fold(
									database::activity::EmoteSettingsChange::default(),
									|sum, c| database::activity::EmoteSettingsChange {
										public_listed: sum.public_listed.or(c.listed),
										approved_personal: sum.approved_personal.or(c.allow_personal),
										..Default::default()
									},
								);
								Some(database::activity::EmoteActivityData::ChangeSettings { old, new })
							}
							AuditLogChange::NewEmoteId(emote_id) => emote_id
								.new
								.into_inner()
								.map(|id| database::activity::EmoteActivityData::Merge { new_emote_id: id.into() }),
							AuditLogChange::Tags(tags) => Some(database::activity::EmoteActivityData::ChangeTags {
								new: tags.new,
								old: tags.old,
							}),
							AuditLogChange::Flags(flags) => {
								let mut old = database::activity::EmoteSettingsChange::default();
								if flags.old.contains(EmoteFlagsModel::Sexual) {
									old.nsfw = Some(true);
								}
								if flags.old.contains(EmoteFlagsModel::Private) {
									old.private = Some(true);
								}
								if flags.old.contains(EmoteFlagsModel::ZeroWidth) {
									old.default_zero_width = Some(true);
								}

								let mut new = database::activity::EmoteSettingsChange::default();
								if flags.new.contains(EmoteFlagsModel::Sexual) {
									new.nsfw = Some(true);
								}
								if flags.new.contains(EmoteFlagsModel::Private) {
									new.private = Some(true);
								}
								if flags.new.contains(EmoteFlagsModel::ZeroWidth) {
									new.default_zero_width = Some(true);
								}

								Some(database::activity::EmoteActivityData::ChangeSettings { old, new })
							}
							AuditLogChange::Owner(owner) => {
								if let (Some(old), Some(new)) = (owner.old.into_inner(), owner.new.into_inner()) {
									Some(database::activity::EmoteActivityData::ChangeOwner {
										old: old.into(),
										new: new.into(),
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
					let activity = database::activity::EmoteActivity {
						emote_id: EmoteId::from(audit_log.target_id).as_uuid(),
						actor_id: Some(UserId::from(audit_log.actor_id).as_uuid()),
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
					AuditLogKind::CreateEmoteSet => database::activity::EmoteSetActivityKind::Create,
					AuditLogKind::UpdateEmoteSet => database::activity::EmoteSetActivityKind::Edit,
					AuditLogKind::DeleteEmoteSet => database::activity::EmoteSetActivityKind::Delete,
					_ => unreachable!(),
				};

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.map(|c| match c {
						AuditLogChange::Name(names) => database::activity::EmoteSetActivityData::ChangeName {
							old: names.old,
							new: names.new,
						},
						AuditLogChange::EmoteSetCapacity(c) => {
							let old = database::activity::EmoteSetSettingsChange {
								capacity: Some(c.old as u32),
								..Default::default()
							};
							let new = database::activity::EmoteSetSettingsChange {
								capacity: Some(c.new as u32),
								..Default::default()
							};
							database::activity::EmoteSetActivityData::ChangeSettings { old, new }
						}
						AuditLogChange::EmoteSetEmotes(emotes) => {
							let added = emotes.added.into_iter().filter_map(|e| e.id).map(|id| id.into()).collect();
							let removed = emotes.removed.into_iter().filter_map(|e| e.id).map(|id| id.into()).collect();
							database::activity::EmoteSetActivityData::ChangeEmotes { added, removed }
						}
						_ => unimplemented!(),
					})
					.map(Some)
					.collect();

				if changes.is_empty() {
					changes.push(None);
				}

				for data in changes {
					let activity = database::activity::EmoteSetActivity {
						emote_set_id: EmoteSetId::from(audit_log.target_id).as_uuid(),
						actor_id: Some(UserId::from(audit_log.actor_id).as_uuid()),
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
					AuditLogKind::CreateUser => database::activity::UserActivityKind::Register,
					AuditLogKind::EditUser => database::activity::UserActivityKind::Edit,
					AuditLogKind::DeleteUser => database::activity::UserActivityKind::Delete,
					AuditLogKind::BanUser => database::activity::UserActivityKind::Ban,
					AuditLogKind::UnbanUser => database::activity::UserActivityKind::Unban,
					_ => unreachable!(),
				};

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.map(|c| match c {
						AuditLogChange::UserEditors(editors) => database::activity::UserActivityData::ChangeEditors {
							added: editors.added.into_iter().filter_map(|e| e.id).map(|id| id.into()).collect(),
							removed: editors.removed.into_iter().filter_map(|e| e.id).map(|id| id.into()).collect(),
						},
						AuditLogChange::UserRoles(roles) => database::activity::UserActivityData::ChangeRoles {
							added: roles.added.into_iter().flatten().map(|id| id.into()).collect(),
							removed: roles.removed.into_iter().flatten().map(|id| id.into()).collect(),
						},
						_ => unimplemented!(),
					})
					.map(Some)
					.collect();

				if changes.is_empty() {
					changes.push(None);
				}

				for data in changes {
					let activity = database::activity::UserActivity {
						user_id: UserId::from(audit_log.target_id).as_uuid(),
						actor_id: Some(UserId::from(audit_log.actor_id).as_uuid()),
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
					AuditLogKind::CreateReport => database::activity::TicketActivityKind::Create,
					AuditLogKind::UpdateReport => database::activity::TicketActivityKind::Edit,
					_ => unreachable!(),
				};

				let mut changes: Vec<_> = audit_log
					.changes
					.into_iter()
					.map(|c| match c {
						AuditLogChange::ReportStatus(status) => database::activity::TicketActivityData::ChangeOpen {
							old: status.old == ReportStatus::Open,
							new: status.new == ReportStatus::Open,
						},
						AuditLogChange::ReportAssignees(assignees) => {
							database::activity::TicketActivityData::ChangeAssignees {
								added: assignees.added.into_iter().map(|id| id.into()).collect(),
								removed: assignees.removed.into_iter().map(|id| id.into()).collect(),
							}
						}
						_ => unimplemented!(),
					})
					.map(Some)
					.collect();

				if changes.is_empty() {
					changes.push(None);
				}

				for data in changes {
					let activity = database::activity::TicketActivity {
						ticket_id: TicketId::from(audit_log.target_id).as_uuid(),
						actor_id: Some(UserId::from(audit_log.actor_id).as_uuid()),
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
			renew_writer(&self.global, &mut self.emote_activity_writer, "emote_activities").await;
			renew_writer(&self.global, &mut self.emote_set_activity_writer, "emote_set_activities").await;
			renew_writer(&self.global, &mut self.user_activity_writer, "user_activities").await;
			renew_writer(&self.global, &mut self.ticket_activity_writer, "ticket_activities").await;
			self.i = 0;
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		if let Err(e) = self.emote_activity_writer.end().await {
			outcome.errors.push(e.into());
		}
		if let Err(e) = self.emote_set_activity_writer.end().await {
			outcome.errors.push(e.into());
		}
		if let Err(e) = self.user_activity_writer.end().await {
			outcome.errors.push(e.into());
		}
		if let Err(e) = self.ticket_activity_writer.end().await {
			outcome.errors.push(e.into());
		}

		outcome
	}
}
