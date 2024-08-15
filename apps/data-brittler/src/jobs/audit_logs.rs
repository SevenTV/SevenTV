use std::mem;
use std::sync::Arc;

use shared::database::emote::EmoteId;
use shared::database::entitlement::{EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::user::ban::UserBanId;
use shared::database::user::editor::UserEditorId;
use shared::database::user::UserId;
use shared::database::{self, stored_event, MongoCollection};
use shared::old_types::EmoteFlagsModel;

use super::{Job, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{self, AuditLogChange, AuditLogChangeArray, AuditLogKind, ReportStatus};

const BATCH_SIZE: usize = 1_000_000;

pub struct AuditLogsJob {
	global: Arc<Global>,
	events: Vec<stored_event::StoredEvent>,
}

impl Job for AuditLogsJob {
	type T = types::AuditLog;

	const NAME: &'static str = "transfer_audit_logs";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating events collection");

			stored_event::StoredEvent::collection(global.target_db()).drop().await?;
			let indexes = stored_event::StoredEvent::indexes();
			if !indexes.is_empty() {
				stored_event::StoredEvent::collection(global.target_db())
					.create_indexes(indexes)
					.await?;
			}
		}

		Ok(Self { global, events: vec![] })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("audit_logs")
	}

	async fn process(&mut self, audit_log: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let mut data = vec![];

		match audit_log.kind {
			AuditLogKind::CreateEmote => data.push(stored_event::StoredEventData::Emote {
				target_id: audit_log.target_id.into(),
				data: stored_event::StoredEventEmoteData::Upload,
			}),
			AuditLogKind::ProcessEmote => {
				data.push(stored_event::StoredEventData::Emote {
					target_id: audit_log.target_id.into(),
					data: stored_event::StoredEventEmoteData::Process {
						event: stored_event::ImageProcessorEvent::Success(None),
					},
				});
			}
			AuditLogKind::UpdateEmote => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::Name(names) => {
							data.push(stored_event::StoredEventData::Emote {
								target_id: audit_log.target_id.into(),
								data: stored_event::StoredEventEmoteData::ChangeName {
									old: names.old,
									new: names.new,
								},
							});
						}
						AuditLogChange::NewEmoteId(change) => {
							if let Some(new_emote_id) = change.new.into_inner() {
								data.push(stored_event::StoredEventData::Emote {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventEmoteData::Merge {
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
							data.push(stored_event::StoredEventData::Emote {
								target_id: audit_log.target_id.into(),
								data: stored_event::StoredEventEmoteData::ChangeFlags { old, new },
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

							data.push(stored_event::StoredEventData::Emote {
								target_id: audit_log.target_id.into(),
								data: stored_event::StoredEventEmoteData::ChangeFlags { old, new },
							});
						}
						AuditLogChange::Tags(tags) => data.push(stored_event::StoredEventData::Emote {
							target_id: audit_log.target_id.into(),
							data: stored_event::StoredEventEmoteData::ChangeTags {
								new: tags.new,
								old: tags.old,
							},
						}),
						AuditLogChange::Owner(owner) => {
							if let (Some(old), Some(new)) = (owner.old.into_inner(), owner.new.into_inner()) {
								data.push(stored_event::StoredEventData::Emote {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventEmoteData::ChangeOwner {
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
			AuditLogKind::DeleteEmote => data.push(stored_event::StoredEventData::Emote {
				target_id: audit_log.target_id.into(),
				data: stored_event::StoredEventEmoteData::Delete,
			}),
			AuditLogKind::CreateEmoteSet => data.push(stored_event::StoredEventData::EmoteSet {
				target_id: audit_log.target_id.into(),
				data: stored_event::StoredEventEmoteSetData::Create,
			}),
			AuditLogKind::DeleteEmoteSet => data.push(stored_event::StoredEventData::EmoteSet {
				target_id: audit_log.target_id.into(),
				data: stored_event::StoredEventEmoteSetData::Delete,
			}),
			AuditLogKind::UpdateEmoteSet => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::Name(names) => data.push(stored_event::StoredEventData::EmoteSet {
							target_id: audit_log.target_id.into(),
							data: stored_event::StoredEventEmoteSetData::ChangeName {
								old: names.old,
								new: names.new,
							},
						}),
						AuditLogChange::EmoteSetCapacity(c) => {
							if c.old != c.new {
								data.push(stored_event::StoredEventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventEmoteSetData::ChangeCapacity {
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
								data.push(stored_event::StoredEventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventEmoteSetData::AddEmote {
										emote_id,
										alias: alias.unwrap_or_default(),
									},
								});
							}
							for emote_id in emotes.removed.into_iter().filter_map(|e| e.id.map(EmoteId::from)) {
								data.push(stored_event::StoredEventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventEmoteSetData::RemoveEmote { emote_id },
								});
							}
							for update in emotes
								.updated
								.into_iter()
								.filter(|e| e.new.id.is_some() && e.old.id.is_some())
								.filter(|e| e.old.id == e.new.id && e.old.name != e.new.name)
							{
								let emote_id = update.new.id.unwrap().into();

								data.push(stored_event::StoredEventData::EmoteSet {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventEmoteSetData::RenameEmote {
										emote_id,
										old_alias: update.old.name.unwrap(),
										new_alias: update.new.name.unwrap(),
									},
								});
							}
						}
						_ => unimplemented!(),
					}
				}
			}
			AuditLogKind::CreateUser => {
				data.push(stored_event::StoredEventData::User {
					target_id: audit_log.target_id.into(),
					data: stored_event::StoredEventUserData::Create,
				});
			}
			AuditLogKind::EditUser => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::UserEditors(editors) => {
							for editor in editors.added.into_iter().filter_map(|e| e.id) {
								let editor_id = UserEditorId {
									user_id: audit_log.target_id.into(),
									editor_id: editor.into(),
								};

								data.push(stored_event::StoredEventData::UserEditor {
									target_id: editor_id,
									data: stored_event::StoredEventUserEditorData::AddEditor {
										editor_id: editor_id.editor_id,
									},
								});
							}
							for editor in editors.removed.into_iter().filter_map(|e| e.id) {
								let editor_id = UserEditorId {
									user_id: audit_log.target_id.into(),
									editor_id: editor.into(),
								};

								data.push(stored_event::StoredEventData::UserEditor {
									target_id: editor_id,
									data: stored_event::StoredEventUserEditorData::RemoveEditor {
										editor_id: editor_id.editor_id,
									},
								});
							}
						}
						AuditLogChange::UserRoles(roles) => {
							for role in roles.added.into_iter().flatten() {
								let entitlement_edge_id = EntitlementEdgeId {
									from: EntitlementEdgeKind::User {
										user_id: audit_log.target_id.into(),
									},
									to: EntitlementEdgeKind::Role { role_id: role.into() },
									managed_by: None,
								};

								data.push(stored_event::StoredEventData::EntitlementEdge {
									target_id: entitlement_edge_id,
									data: stored_event::StoredEventEntitlementEdgeData::Create,
								});
							}
							for role in roles.removed.into_iter().flatten() {
								let entitlement_edge_id = EntitlementEdgeId {
									from: EntitlementEdgeKind::User {
										user_id: audit_log.target_id.into(),
									},
									to: EntitlementEdgeKind::Role { role_id: role.into() },
									managed_by: None,
								};

								data.push(stored_event::StoredEventData::EntitlementEdge {
									target_id: entitlement_edge_id,
									data: stored_event::StoredEventEntitlementEdgeData::Delete,
								});
							}
						}
						_ => unimplemented!(),
					}
				}
			}
			AuditLogKind::DeleteUser => data.push(stored_event::StoredEventData::User {
				target_id: audit_log.target_id.into(),
				data: stored_event::StoredEventUserData::Delete,
			}),
			AuditLogKind::BanUser => data.push(stored_event::StoredEventData::UserBan {
				target_id: UserBanId::nil(), // TODO: how should we know this?
				data: stored_event::StoredEventUserBanData::Ban,
			}),
			AuditLogKind::UnbanUser => data.push(stored_event::StoredEventData::UserBan {
				target_id: UserBanId::nil(), // TODO: how should we know this?
				data: stored_event::StoredEventUserBanData::Unban,
			}),
			AuditLogKind::CreateReport => data.push(stored_event::StoredEventData::Ticket {
				target_id: audit_log.target_id.into(),
				data: stored_event::StoredEventTicketData::Create,
			}),
			AuditLogKind::UpdateReport => {
				for change in audit_log.changes {
					match change {
						AuditLogChange::ReportStatus(status) => {
							let old = status.old == ReportStatus::Open;
							let new = status.new == ReportStatus::Open;

							if new != old {
								data.push(stored_event::StoredEventData::Ticket {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventTicketData::ChangeOpen { old, new },
								});
							}
						}
						AuditLogChange::ReportAssignees(assignees) => {
							for member in assignees.added {
								data.push(stored_event::StoredEventData::Ticket {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventTicketData::AddMember { member: member.into() },
								});
							}

							for member in assignees.removed {
								data.push(stored_event::StoredEventData::Ticket {
									target_id: audit_log.target_id.into(),
									data: stored_event::StoredEventTicketData::RemoveMember { member: member.into() },
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
			self.events.push(stored_event::StoredEvent {
				id: stored_event::StoredEventId::with_timestamp(audit_log.id.timestamp().to_chrono()),
				actor_id: Some(UserId::from(audit_log.actor_id)),
				data,
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			});
		}

		if self.events.len() >= BATCH_SIZE {
			match stored_event::StoredEvent::collection(self.global.target_db())
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
		match stored_event::StoredEvent::collection(self.global.target_db())
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
