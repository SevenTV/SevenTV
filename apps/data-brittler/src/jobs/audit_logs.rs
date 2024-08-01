use std::mem;
use std::sync::Arc;

use shared::database::emote::EmoteId;
use shared::database::entitlement::{EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::user::ban::UserBanId;
use shared::database::user::editor::UserEditorId;
use shared::database::user::UserId;
use shared::database::{self, event, MongoCollection};
use shared::old_types::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, AuditLogChange, AuditLogChangeArray, AuditLogKind, ReportStatus};

const BATCH_SIZE: usize = 1_000_000;

pub struct AuditLogsJob {
	global: Arc<Global>,
	events: Vec<database::event::Event>,
}

impl Job for AuditLogsJob {
	type T = types::AuditLog;

	const NAME: &'static str = "transfer_audit_logs";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating events collection");

			database::event::Event::collection(global.target_db()).drop().await?;
			let indexes = database::event::Event::indexes();
			if !indexes.is_empty() {
				database::event::Event::collection(global.target_db())
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self {
			global,
			events: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("audit_logs")
	}

	async fn process(&mut self, audit_log: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let mut data = vec![];

		match audit_log.kind {
			AuditLogKind::CreateEmote => data.push(database::event::EventData::Emote {
				target_id: audit_log.target_id.into(),
				data: database::event::EventEmoteData::Upload,
			}),
			AuditLogKind::ProcessEmote => data.push(database::event::EventData::Emote {
				target_id: audit_log.target_id.into(),
				data: database::event::EventEmoteData::Process { outcome: event::ImageProcessOutcome::Success },
			}),
			AuditLogKind::UpdateEmote => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::Name(names) => {
							data.push(database::event::EventData::Emote {
								target_id: audit_log.target_id.into(),
								data: database::event::EventEmoteData::ChangeName {
									old: names.old,
									new: names.new,
								},
							});
						}
						AuditLogChange::NewEmoteId(change) => {
							if let Some(new_emote_id) = change.new.into_inner() {
								data.push(database::event::EventData::Emote {
									target_id: audit_log.target_id.into(),
									data: database::event::EventEmoteData::Merge {
										new_emote_id: new_emote_id.into(),
									},
								});
							}
						}
						AuditLogChange::EmoteVersions(AuditLogChangeArray { updated, .. }) => {
							let old = updated
								.iter()
								.map(|u| &u.old)
								.fold(database::emote::EmoteFlags::default(), |sum, c| {
									sum | database::emote::EmoteFlags::from(c)
								});
							let new = updated
								.iter()
								.map(|u| &u.new)
								.fold(database::emote::EmoteFlags::default(), |sum, c| {
									sum | database::emote::EmoteFlags::from(c)
								});
							data.push(database::event::EventData::Emote {
								target_id: audit_log.target_id.into(),
								data: database::event::EventEmoteData::ChangeFlags { old, new },
							});
						}
						AuditLogChange::Flags(flags) => {
							let mut old = database::emote::EmoteFlags::none();
							if flags.old.contains(EmoteFlagsModel::Sexual) {
								old |= database::emote::EmoteFlags::Nsfw;
							}
							if flags.old.contains(EmoteFlagsModel::Private) {
								old |= database::emote::EmoteFlags::Private;
							}
							if flags.old.contains(EmoteFlagsModel::ZeroWidth) {
								old |= database::emote::EmoteFlags::DefaultZeroWidth;
							}

							let mut new = database::emote::EmoteFlags::none();
							if flags.new.contains(EmoteFlagsModel::Sexual) {
								new |= database::emote::EmoteFlags::Nsfw;
							}
							if flags.new.contains(EmoteFlagsModel::Private) {
								new |= database::emote::EmoteFlags::Private;
							}
							if flags.new.contains(EmoteFlagsModel::ZeroWidth) {
								new |= database::emote::EmoteFlags::DefaultZeroWidth;
							}

							data.push(database::event::EventData::Emote {
								target_id: audit_log.target_id.into(),
								data: database::event::EventEmoteData::ChangeFlags { old, new },
							});
						}
						AuditLogChange::Tags(tags) => data.push(database::event::EventData::Emote {
							target_id: audit_log.target_id.into(),
							data: database::event::EventEmoteData::ChangeTags {
								new: tags.new,
								old: tags.old,
							},
						}),
						AuditLogChange::Owner(owner) => {
							if let (Some(old), Some(new)) = (owner.old.into_inner(), owner.new.into_inner()) {
								data.push(database::event::EventData::Emote {
									target_id: audit_log.target_id.into(),
									data: database::event::EventEmoteData::ChangeOwner {
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
			AuditLogKind::DeleteEmote => data.push(database::event::EventData::Emote {
				target_id: audit_log.target_id.into(),
				data: database::event::EventEmoteData::Delete,
			}),
			AuditLogKind::CreateEmoteSet => data.push(database::event::EventData::EmoteSet {
				target_id: audit_log.target_id.into(),
				data: database::event::EventEmoteSetData::Create,
			}),
			AuditLogKind::DeleteEmoteSet => data.push(database::event::EventData::EmoteSet {
				target_id: audit_log.target_id.into(),
				data: database::event::EventEmoteSetData::Delete,
			}),
			AuditLogKind::UpdateEmoteSet => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::Name(names) => data.push(database::event::EventData::EmoteSet {
							target_id: audit_log.target_id.into(),
							data: database::event::EventEmoteSetData::ChangeName {
								old: names.old,
								new: names.new,
							},
						}),
						AuditLogChange::EmoteSetCapacity(c) => {
							if c.old != c.new {
								data.push(database::event::EventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: database::event::EventEmoteSetData::ChangeCapacity {
										old: Some(c.old),
										new: Some(c.new),
									},
								});
							}
						}
						AuditLogChange::EmoteSetEmotes(emotes) => {
							for (emote_id, alias) in emotes
								.added
								.into_iter()
								.filter_map(|e| e.id.map(|id| (EmoteId::from(id), e.name)))
							{
								data.push(database::event::EventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: event::EventEmoteSetData::AddEmote {
										emote_id,
										alias: alias.unwrap_or_default(),
									},
								});
							}
							for emote_id in emotes.removed.into_iter().filter_map(|e| e.id.map(EmoteId::from)) {
								data.push(database::event::EventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: event::EventEmoteSetData::RemoveEmote { emote_id },
								});
							}
							for update in emotes
								.updated
								.into_iter()
								.filter(|e| e.new.id.is_some() && e.old.id.is_some())
								.filter(|e| e.old.id == e.new.id && e.old.name != e.new.name)
							{
								let emote_id = update.new.id.unwrap().into();

								data.push(database::event::EventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: event::EventEmoteSetData::RenameEmote {
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
			// we don't need this event because we can see when the user was created using the user id
			AuditLogKind::CreateUser => {}
			AuditLogKind::EditUser => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::UserEditors(editors) => {
							for editor in editors.added.into_iter().filter_map(|e| e.id) {
								let editor_id = UserEditorId {
									user_id: audit_log.target_id.into(),
									editor_id: editor.into(),
								};

								data.push(database::event::EventData::UserEditor {
									target_id: editor_id,
									data: database::event::EventUserEditorData::AddEditor {
										editor_id: editor_id.editor_id,
									},
								});
							}
							for editor in editors.removed.into_iter().filter_map(|e| e.id) {
								let editor_id = UserEditorId {
									user_id: audit_log.target_id.into(),
									editor_id: editor.into(),
								};

								data.push(database::event::EventData::UserEditor {
									target_id: editor_id,
									data: database::event::EventUserEditorData::RemoveEditor {
										editor_id: editor_id.editor_id,
									},
								});
							}
						}
						AuditLogChange::UserRoles(roles) => {
							for role in roles.added.into_iter().flatten() {
								let entitlement_edge_id = EntitlementEdgeId {
									from: EntitlementEdgeKind::User { user_id: audit_log.target_id.into() },
									to: EntitlementEdgeKind::Role { role_id: role.into() },
									managed_by: None,
								};

								data.push(database::event::EventData::EntitlementEdge { target_id: entitlement_edge_id, data: event::EventEntitlementEdgeData::Create });
							}
							for role in roles.removed.into_iter().flatten() {
								let entitlement_edge_id = EntitlementEdgeId {
									from: EntitlementEdgeKind::User { user_id: audit_log.target_id.into() },
									to: EntitlementEdgeKind::Role { role_id: role.into() },
									managed_by: None,
								};

								data.push(database::event::EventData::EntitlementEdge { target_id: entitlement_edge_id, data: event::EventEntitlementEdgeData::Delete });
							}
						}
						_ => unimplemented!(),
					}
				}
			}
			AuditLogKind::DeleteUser => data.push(database::event::EventData::User {
				target_id: audit_log.target_id.into(),
				data: database::event::EventUserData::Delete,
			}),
			AuditLogKind::BanUser => data.push(database::event::EventData::UserBan {
				target_id: UserBanId::nil(), // TODO: how should we know this?
				data: database::event::EventUserBanData::Ban,
			}),
			AuditLogKind::UnbanUser => data.push(database::event::EventData::UserBan {
				target_id: UserBanId::nil(), // TODO: how should we know this?
				data: database::event::EventUserBanData::Unban,
			}),
			AuditLogKind::CreateReport => data.push(database::event::EventData::Ticket {
				target_id: audit_log.target_id.into(),
				data: database::event::EventTicketData::Create,
			}),
			AuditLogKind::UpdateReport => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::ReportStatus(status) => {
							let old = status.old == ReportStatus::Open;
							let new = status.new == ReportStatus::Open;

							if new != old {
								data.push(database::event::EventData::Ticket {
									target_id: audit_log.target_id.into(),
									data: database::event::EventTicketData::ChangeOpen { old, new },
								});
							}
						}
						AuditLogChange::ReportAssignees(assignees) => {
							for member in assignees.added {
								data.push(database::event::EventData::Ticket {
									target_id: audit_log.target_id.into(),
									data: database::event::EventTicketData::AddMember { member: member.into() },
								});
							}

							for member in assignees.removed {
								data.push(database::event::EventData::Ticket {
									target_id: audit_log.target_id.into(),
									data: database::event::EventTicketData::RemoveMember { member: member.into() },
								});
							}
						}
						_ => unimplemented!(),
					}
				}
			}
			k => outcome.errors.push(error::Error::UnsupportedAuditLogKind(k)),
		}

		for data in data {
			self.events.push(database::event::Event {
				id: database::event::EventId::with_timestamp(audit_log.id.timestamp().to_chrono()),
				actor_id: Some(UserId::from(audit_log.actor_id)),
				data,
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			});
		}

		if self.events.len() >= BATCH_SIZE {
			match database::event::Event::collection(self.global.target_db())
				.insert_many(mem::take(&mut self.events))
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
		match database::event::Event::collection(self.global.target_db())
			.insert_many(&self.events)
			.await
		{
			Ok(res) => {
				if res.inserted_ids.len() != self.events.len() {
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
