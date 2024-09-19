use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::emote::{Emote, EmoteId};
use shared::database::emote_set::{EmoteSet, EmoteSetId};
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::role::{Role, RoleId};
use shared::database::ticket::{Ticket, TicketId};
use shared::database::user::editor::UserEditorId;
use shared::database::user::{User, UserId};
use shared::database::{self, stored_event};
use shared::old_types::EmoteFlagsModel;

use super::{JobOutcome, ProcessOutcome};
use crate::error;
use crate::global::Global;
use crate::types::{AuditLog, AuditLogChange, AuditLogChangeArray, AuditLogKind, ReportStatus};

struct ProcessInput<'a> {
	pub audit_log: AuditLog,
	pub stats: &'a mut BTreeMap<(EmoteId, time::Date), i32>,
	pub events: &'a mut Vec<stored_event::StoredEvent>,
	pub emotes: &'a HashMap<EmoteId, Emote>,
	pub users: &'a HashMap<UserId, User>,
	pub emote_sets: &'a HashMap<EmoteSetId, EmoteSet>,
	pub roles: &'a HashMap<RoleId, Role>,
	pub tickets: &'a HashMap<TicketId, Ticket>,
}

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub stats: &'a mut BTreeMap<(EmoteId, time::Date), i32>,
	pub events: &'a mut Vec<stored_event::StoredEvent>,
	pub emotes: &'a HashMap<EmoteId, Emote>,
	pub users: &'a HashMap<UserId, User>,
	pub emote_sets: &'a HashMap<EmoteSetId, EmoteSet>,
	pub roles: &'a HashMap<RoleId, Role>,
	pub tickets: &'a HashMap<TicketId, Ticket>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("audit_logs");

	let RunInput {
		global,
		stats,
		events,
		emotes,
		emote_sets,
		users,
		roles,
		tickets,
	} = input;

	let mut cursor = global
		.main_source_db
		.collection::<AuditLog>("audit_logs")
		.find(bson::doc! {})
		.await
		.context("query")?;

	while let Some(audit_log) = cursor.next().await {
		match audit_log {
			Ok(audit_log) => {
				outcome += process(ProcessInput {
					audit_log,
					stats,
					events,
					emotes,
					emote_sets,
					users,
					roles,
					tickets,
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

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let mut outcome = ProcessOutcome::default();

	let ProcessInput {
		audit_log,
		stats,
		events,
		emotes,
		emote_sets,
		users,
		roles,
		tickets,
	} = input;

	let mut data = vec![];

	match audit_log.kind {
		AuditLogKind::CreateEmote
		| AuditLogKind::ProcessEmote
		| AuditLogKind::UpdateEmote
		| AuditLogKind::MergeEmote
		| AuditLogKind::DeleteEmote => {
			if !emotes.contains_key(&audit_log.target_id.into()) {
				return outcome;
			}
		}
		AuditLogKind::CreateEmoteSet | AuditLogKind::UpdateEmoteSet | AuditLogKind::DeleteEmoteSet => {
			if !emote_sets.contains_key(&audit_log.target_id.into()) {
				return outcome;
			}
		}
		AuditLogKind::CreateUser | AuditLogKind::EditUser | AuditLogKind::DeleteUser => {
			if !users.contains_key(&audit_log.target_id.into()) {
				return outcome;
			}
		}
		AuditLogKind::CreateReport | AuditLogKind::UpdateReport => {
			if !tickets.contains_key(&audit_log.target_id.into()) {
				return outcome;
			}
		}
		_ => {}
	}

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
					AuditLogChange::EmoteSetEmotes(changed_emotes) => {
						for (emote_id, alias) in changed_emotes
							.added
							.into_iter()
							.filter_map(|e| e.id.map(|id| (EmoteId::from(id), e.name)))
						{
							if !emotes.contains_key(&emote_id) {
								continue;
							}

							data.push(stored_event::StoredEventData::EmoteSet {
								target_id: audit_log.target_id.into(),
								data: stored_event::StoredEventEmoteSetData::AddEmote {
									emote_id,
									alias: alias.unwrap_or_default(),
								},
							});

							*stats
								.entry((emote_id, audit_log.id.timestamp().to_time_0_3().date()))
								.or_default() += 1;
						}
						for emote_id in changed_emotes.removed.into_iter().filter_map(|e| e.id.map(EmoteId::from)) {
							if !emotes.contains_key(&emote_id) {
								continue;
							}

							data.push(stored_event::StoredEventData::EmoteSet {
								target_id: audit_log.target_id.into(),
								data: stored_event::StoredEventEmoteSetData::RemoveEmote { emote_id },
							});

							*stats
								.entry((emote_id, audit_log.id.timestamp().to_time_0_3().date()))
								.or_default() -= 1;
						}
						for update in changed_emotes
							.updated
							.into_iter()
							.filter(|e| e.new.id.is_some() && e.old.id.is_some())
							.filter(|e| e.old.id == e.new.id && e.old.name != e.new.name)
						{
							let emote_id = update.new.id.unwrap().into();
							if !emotes.contains_key(&emote_id) {
								continue;
							}

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

							if !users.contains_key(&editor_id.editor_id) {
								continue;
							}

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

							if !users.contains_key(&editor_id.editor_id) {
								continue;
							}

							data.push(stored_event::StoredEventData::UserEditor {
								target_id: editor_id,
								data: stored_event::StoredEventUserEditorData::RemoveEditor {
									editor_id: editor_id.editor_id,
								},
							});
						}
					}
					AuditLogChange::UserRoles(changed_roles) => {
						for role in changed_roles.added.into_iter().flatten() {
							if !roles.contains_key(&role.into()) {
								continue;
							}

							data.push(stored_event::StoredEventData::User {
								target_id: audit_log.target_id.into(),
								data: stored_event::StoredEventUserData::AddEntitlement {
									target: EntitlementEdgeKind::Role { role_id: role.into() },
								},
							});
						}
						for role in changed_roles.removed.into_iter().flatten() {
							if !roles.contains_key(&role.into()) {
								continue;
							}

							data.push(stored_event::StoredEventData::User {
								target_id: audit_log.target_id.into(),
								data: stored_event::StoredEventUserData::RemoveEntitlement {
									target: EntitlementEdgeKind::Role { role_id: role.into() },
								},
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

	events.extend(data.into_iter().map(|data| stored_event::StoredEvent {
		id: stored_event::StoredEventId::with_timestamp(audit_log.id.timestamp().to_chrono()),
		actor_id: Some(UserId::from(audit_log.actor_id)),
		session_id: None,
		data,
		updated_at: chrono::Utc::now(),
		search_updated_at: None,
	}));

	outcome
}
