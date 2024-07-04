use std::mem;
use std::sync::Arc;

use shared::database::emote::EmoteId;
use shared::database::user::UserId;
use shared::database::{self, audit_log, Collection};
use shared::old_types::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, AuditLogChange, AuditLogChangeArray, AuditLogKind, ReportStatus};

const BATCH_SIZE: usize = 1_000_000;

pub struct AuditLogsJob {
	global: Arc<Global>,
	audit_logs: Vec<database::audit_log::AuditLog>,
}

impl Job for AuditLogsJob {
	type T = types::AuditLog;

	const NAME: &'static str = "transfer_audit_logs";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating audit_logs collection");

			database::audit_log::AuditLog::collection(global.target_db()).drop().await?;
			let indexes = database::audit_log::AuditLog::indexes();
			if !indexes.is_empty() {
				database::audit_log::AuditLog::collection(global.target_db())
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self {
			global,
			audit_logs: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("audit_logs")
	}

	async fn process(&mut self, audit_log: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let mut data = vec![];

		match audit_log.kind {
			AuditLogKind::CreateEmote => data.push(database::audit_log::AuditLogData::Emote {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogEmoteData::Upload,
			}),
			AuditLogKind::ProcessEmote => data.push(database::audit_log::AuditLogData::Emote {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogEmoteData::Process,
			}),
			AuditLogKind::UpdateEmote => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::Name(names) => {
							data.push(database::audit_log::AuditLogData::Emote {
								target_id: audit_log.target_id.into(),
								data: database::audit_log::AuditLogEmoteData::ChangeName {
									old: names.old,
									new: names.new,
								},
							});
						}
						AuditLogChange::NewEmoteId(change) => {
							if let Some(new_emote_id) = change.new.into_inner() {
								data.push(database::audit_log::AuditLogData::Emote {
									target_id: audit_log.target_id.into(),
									data: database::audit_log::AuditLogEmoteData::Merge {
										new_emote_id: new_emote_id.into(),
									},
								});
							}
						}
						AuditLogChange::EmoteVersions(AuditLogChangeArray { updated, .. }) => {
							let old = updated.iter().map(|u| &u.old).fold(
								database::audit_log::EmoteSettingsChange::default(),
								|sum, c| database::audit_log::EmoteSettingsChange {
									public_listed: sum.public_listed.or(c.listed),
									approved_personal: sum.approved_personal.or(c.allow_personal),
									..Default::default()
								},
							);
							let new = updated.iter().map(|u| &u.new).fold(
								database::audit_log::EmoteSettingsChange::default(),
								|sum, c| database::audit_log::EmoteSettingsChange {
									public_listed: sum.public_listed.or(c.listed),
									approved_personal: sum.approved_personal.or(c.allow_personal),
									..Default::default()
								},
							);
							data.push(database::audit_log::AuditLogData::Emote {
								target_id: audit_log.target_id.into(),
								data: database::audit_log::AuditLogEmoteData::ChangeSettings { old, new },
							});
						}
						AuditLogChange::Flags(flags) => {
							let mut old = database::audit_log::EmoteSettingsChange::default();
							if flags.old.contains(EmoteFlagsModel::Sexual) {
								old.nsfw = Some(true);
							}
							if flags.old.contains(EmoteFlagsModel::Private) {
								old.private = Some(true);
							}
							if flags.old.contains(EmoteFlagsModel::ZeroWidth) {
								old.default_zero_width = Some(true);
							}

							let mut new = database::audit_log::EmoteSettingsChange::default();
							if flags.new.contains(EmoteFlagsModel::Sexual) {
								new.nsfw = Some(true);
							}
							if flags.new.contains(EmoteFlagsModel::Private) {
								new.private = Some(true);
							}
							if flags.new.contains(EmoteFlagsModel::ZeroWidth) {
								new.default_zero_width = Some(true);
							}

							data.push(database::audit_log::AuditLogData::Emote {
								target_id: audit_log.target_id.into(),
								data: database::audit_log::AuditLogEmoteData::ChangeSettings { old, new },
							});
						}
						AuditLogChange::Tags(tags) => data.push(database::audit_log::AuditLogData::Emote {
							target_id: audit_log.target_id.into(),
							data: database::audit_log::AuditLogEmoteData::ChangeTags {
								new: tags.new,
								old: tags.old,
							},
						}),
						AuditLogChange::Owner(owner) => {
							if let (Some(old), Some(new)) = (owner.old.into_inner(), owner.new.into_inner()) {
								data.push(database::audit_log::AuditLogData::Emote {
									target_id: audit_log.target_id.into(),
									data: database::audit_log::AuditLogEmoteData::ChangeOwner {
										old: old.into(),
										new: new.into(),
									},
								});
							} else {
								// TODO: do something here?
							}
						}
						_ => unimplemented!(),
					}
				}
			}
			// we don't know what it got merged into
			AuditLogKind::MergeEmote => {}
			AuditLogKind::DeleteEmote => data.push(database::audit_log::AuditLogData::Emote {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogEmoteData::Delete,
			}),
			AuditLogKind::CreateEmoteSet => data.push(database::audit_log::AuditLogData::EmoteSet {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogEmoteSetData::Create,
			}),
			AuditLogKind::DeleteEmoteSet => data.push(database::audit_log::AuditLogData::EmoteSet {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogEmoteSetData::Delete,
			}),
			AuditLogKind::UpdateEmoteSet => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::Name(names) => data.push(database::audit_log::AuditLogData::EmoteSet {
							target_id: audit_log.target_id.into(),
							data: database::audit_log::AuditLogEmoteSetData::ChangeName {
								old: names.old,
								new: names.new,
							},
						}),
						AuditLogChange::EmoteSetCapacity(c) => {
							let old = database::audit_log::EmoteSetSettingsChange {
								capacity: Some(c.old as u32),
								..Default::default()
							};
							let new = database::audit_log::EmoteSetSettingsChange {
								capacity: Some(c.new as u32),
								..Default::default()
							};
							data.push(database::audit_log::AuditLogData::EmoteSet {
								target_id: audit_log.target_id.into(),
								data: database::audit_log::AuditLogEmoteSetData::ChangeSettings { old, new },
							});
						}
						AuditLogChange::EmoteSetEmotes(emotes) => {
							for emote_id in emotes.added.into_iter().filter_map(|e| e.id.map(EmoteId::from)) {
								data.push(database::audit_log::AuditLogData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: audit_log::AuditLogEmoteSetData::AddEmote { emote_id },
								});
							}
							for emote_id in emotes.removed.into_iter().filter_map(|e| e.id.map(EmoteId::from)) {
								data.push(database::audit_log::AuditLogData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: audit_log::AuditLogEmoteSetData::RemoveEmote { emote_id },
								});
							}
							for update in emotes
								.updated
								.into_iter()
								.filter(|e| e.new.id.is_some() && e.old.id.is_some())
								.filter(|e| e.old.id == e.new.id && e.old.name != e.new.name)
							{
								let emote_id = update.new.id.unwrap().into();

								data.push(database::audit_log::AuditLogData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: audit_log::AuditLogEmoteSetData::RenameEmote {
										emote_id,
										old_name: update.old.name.unwrap(),
										new_name: update.new.name.unwrap(),
									},
								});
							}
						}
						_ => unimplemented!(),
					}
				}
			}
			AuditLogKind::CreateUser => data.push(database::audit_log::AuditLogData::User {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogUserData::Register,
			}),
			AuditLogKind::EditUser => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::UserEditors(editors) => {
							for editor in editors.added.into_iter().filter_map(|e| e.id) {
								data.push(database::audit_log::AuditLogData::User {
									target_id: audit_log.target_id.into(),
									data: database::audit_log::AuditLogUserData::AddEditor {
										editor_id: editor.into(),
									},
								});
							}
							for editor in editors.removed.into_iter().filter_map(|e| e.id) {
								data.push(database::audit_log::AuditLogData::User {
									target_id: audit_log.target_id.into(),
									data: database::audit_log::AuditLogUserData::RemoveEditor {
										editor_id: editor.into(),
									},
								});
							}
						}
						AuditLogChange::UserRoles(roles) => {
							for role in roles.added.into_iter().flatten() {
								data.push(database::audit_log::AuditLogData::User {
									target_id: audit_log.target_id.into(),
									data: database::audit_log::AuditLogUserData::AddRole { role_id: role.into() },
								});
							}
							for role in roles.removed.into_iter().flatten() {
								data.push(database::audit_log::AuditLogData::User {
									target_id: audit_log.target_id.into(),
									data: database::audit_log::AuditLogUserData::RemoveRole { role_id: role.into() },
								});
							}
						}
						_ => unimplemented!(),
					}
				}
			}
			AuditLogKind::DeleteUser => data.push(database::audit_log::AuditLogData::User {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogUserData::Delete,
			}),
			AuditLogKind::BanUser => data.push(database::audit_log::AuditLogData::User {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogUserData::Ban,
			}),
			AuditLogKind::UnbanUser => data.push(database::audit_log::AuditLogData::User {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogUserData::Unban,
			}),
			AuditLogKind::CreateReport => data.push(database::audit_log::AuditLogData::Ticket {
				target_id: audit_log.target_id.into(),
				data: database::audit_log::AuditLogTicketData::Create,
			}),
			AuditLogKind::UpdateReport => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::ReportStatus(status) => {
							data.push(database::audit_log::AuditLogData::Ticket {
								target_id: audit_log.target_id.into(),
								data: database::audit_log::AuditLogTicketData::ChangeOpen {
									old: status.old == ReportStatus::Open,
									new: status.new == ReportStatus::Open,
								},
							});
						}
						AuditLogChange::ReportAssignees(assignees) => {
							data.push(database::audit_log::AuditLogData::Ticket {
								target_id: audit_log.target_id.into(),
								data: database::audit_log::AuditLogTicketData::ChangeAssignees {
									added: assignees.added.into_iter().map(|id| id.into()).collect(),
									removed: assignees.removed.into_iter().map(|id| id.into()).collect(),
								},
							});
						}
						_ => unimplemented!(),
					}
				}
			}
			k => outcome.errors.push(error::Error::UnsupportedAuditLogKind(k)),
		}

		for data in data {
			self.audit_logs.push(database::audit_log::AuditLog {
				id: database::audit_log::AuditLogId::with_timestamp(audit_log.id.timestamp().to_chrono()),
				actor_id: Some(UserId::from(audit_log.actor_id)),
				data,
			});
		}

		if self.audit_logs.len() >= BATCH_SIZE {
			match database::audit_log::AuditLog::collection(self.global.target_db())
				.insert_many(mem::take(&mut self.audit_logs))
				.await
			{
				Ok(res) => {
					outcome.inserted_rows += res.inserted_ids.len() as u64;
					if res.inserted_ids.len() < BATCH_SIZE {
						return outcome.with_error(error::Error::InsertMany);
					}
				}
				Err(e) => return outcome.with_error(e),
			}
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		match database::audit_log::AuditLog::collection(self.global.target_db())
			.insert_many(&self.audit_logs)
			.await
		{
			Ok(res) => {
				if res.inserted_ids.len() != self.audit_logs.len() {
					ProcessOutcome::default().with_error(error::Error::InsertMany)
				} else {
					let mut outcome = ProcessOutcome::default();
					outcome.inserted_rows = res.inserted_ids.len() as u64;
					outcome
				}
			}
			Err(e) => ProcessOutcome::default().with_error(e),
		}
	}
}
