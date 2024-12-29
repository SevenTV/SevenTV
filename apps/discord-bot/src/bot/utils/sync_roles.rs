use std::{collections::HashSet, fmt};

use serenity::all::{Http, Member, RoleId};
use shared::database::{role::Role as DatabaseRole, Id};

use crate::global::Global;

pub struct SyncedRoles {
	pub added: Vec<Role>,
	pub removed: Vec<Role>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Role {
	Contributor,
	Subscriber,
}

impl Role {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Contributor => "Contributor",
			Self::Subscriber => "Subscriber",
		}
	}

	fn to_role_id(&self, global: &Global) -> RoleId {
		match self {
			Self::Contributor => RoleId::new(global.config.bot.contributor_role_id),
			Self::Subscriber => RoleId::new(global.config.bot.subscriber_role_id),
		}
	}

	fn from_u64(num: u64, global: &Global) -> Option<Self> {
		if num == global.config.bot.contributor_role_id {
			Some(Role::Contributor)
		} else if num == global.config.bot.subscriber_role_id {
			Some(Role::Subscriber)
		} else {
			None
		}
	}
}

pub enum SyncRolesError {
	FailedAddRoles,
	FailedRemoveRoles,
}

impl fmt::Display for SyncRolesError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::FailedAddRoles => f.write_str("failed to add roles"),
			Self::FailedRemoveRoles => f.write_str("failed to remove roles"),
		}
	}
}

pub async fn sync_roles(
	role_ids: &[&Id<DatabaseRole>],
	member: &Member,
	http: &Http,
	global: &Global,
) -> Result<SyncedRoles, SyncRolesError> {
	let current_roles = HashSet::from_iter(member.roles.iter().filter_map(|r| Role::from_u64(r.get(), &global)));
	let mut needed_roles = HashSet::new();

	for id in role_ids {
		match id.to_string().as_str() {
			// Contributor
			"01F6ZEHYGG0008GVK38JE54RXH" => {
				needed_roles.insert(Role::Contributor);
			}
			// Subscriber
			"01F37R3RFR0000K96678WEQT01" => {
				needed_roles.insert(Role::Subscriber);
			}
			_ => {}
		}
	}

	let added = needed_roles.difference(&current_roles).cloned().collect::<Vec<_>>();
	let removed = current_roles.difference(&needed_roles).cloned().collect::<Vec<_>>();

	if !added.is_empty() {
		member
			.add_roles(http, &added.iter().map(|role| role.to_role_id(&global)).collect::<Vec<_>>())
			.await
			.map_err(|_| SyncRolesError::FailedAddRoles)?;
	}
	if !removed.is_empty() {
		member
			.remove_roles(http, &removed.iter().map(|role| role.to_role_id(&global)).collect::<Vec<_>>())
			.await
			.map_err(|_| SyncRolesError::FailedRemoveRoles)?;
	}

	Ok(SyncedRoles { added, removed })
}
