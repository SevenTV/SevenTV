use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Weak};

use scuffle_foundations::dataloader::{DataLoader, Loader};
use shared::database::entitlement::{CalculatedEntitlements, EntitlementEdgeKind};
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::role::permissions::Permissions;
use shared::database::role::{Role, RoleId};
use shared::database::user::{FullUser, User, UserComputed, UserId};

use super::entitlement_edge::EntitlementEdgeGraphTraverse;
use crate::global::Global;

pub struct FullUserLoader {
	computed_loader: DataLoader<UserComputedLoader>,
}

impl FullUserLoader {
	pub fn new(global: Weak<Global>) -> Self {
		Self {
			computed_loader: UserComputedLoader::new(global.clone()),
		}
	}

	/// Performs a full user load fetching all necessary data using the graph
	pub async fn load(&self, global: &Arc<Global>, user_id: UserId) -> Result<Option<FullUser>, ()> {
		self.load_many(global, std::iter::once(user_id))
			.await
			.map(|mut users| users.remove(&user_id))
	}

	/// Performs a full user load fetching all necessary data using the graph
	pub async fn load_many(
		&self,
		global: &Arc<Global>,
		user_ids: impl IntoIterator<Item = UserId>,
	) -> Result<HashMap<UserId, FullUser>, ()> {
		let users = global.user_by_id_loader().load_many(user_ids).await?;
		self.load_user_many(users.into_values()).await
	}

	/// Performs a full user load fetching all necessary data using the graph
	pub async fn load_user(&self, user: User) -> Result<FullUser, ()> {
		let id = user.id;
		self.load_user_many(std::iter::once(user)).await?.remove(&id).ok_or(())
	}

	/// Performs a full user load fetching all necessary data using the graph
	pub async fn load_user_many(&self, user: impl IntoIterator<Item = User>) -> Result<HashMap<UserId, FullUser>, ()> {
		let users = user.into_iter().collect::<Vec<_>>();

		let computed = self.computed_loader.load_many(users.iter().map(|user| user.id)).await?;

		Ok(users
			.into_iter()
			.filter_map(|user| {
				let Some(mut computed) = computed.get(&user.id).cloned() else {
					return None;
				};

				if let Some(active_bans) = user.active_bans() {
					computed.permissions.merge(active_bans.permissions());
				}

				Some((user.id, FullUser { user, computed }))
			})
			.collect())
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast(
		&self,
		global: &Arc<Global>,
		role_order: &[RoleId],
		user_id: UserId,
	) -> Result<Option<FullUser>, ()> {
		self.load_fast_many(global, role_order, std::iter::once(user_id))
			.await
			.map(|mut users| users.remove(&user_id))
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast_many(
		&self,
		global: &Arc<Global>,
		role_order: &[RoleId],
		user_ids: impl IntoIterator<Item = UserId>,
	) -> Result<HashMap<UserId, FullUser>, ()> {
		let users = global.user_by_id_loader().load_many(user_ids).await?;
		self.load_fast_user_many(global, role_order, users.into_values()).await
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast_user(&self, global: &Arc<Global>, role_order: &[RoleId], user: User) -> Result<FullUser, ()> {
		let id = user.id;
		self.load_fast_user_many(global, role_order, std::iter::once(user))
			.await?
			.remove(&id)
			.ok_or(())
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast_user_many(
		&self,
		global: &Arc<Global>,
		role_order: &[RoleId],
		user: impl IntoIterator<Item = User>,
	) -> Result<HashMap<UserId, FullUser>, ()> {
		let mut role_ids = HashSet::new();

		let mut users = user
			.into_iter()
			.map(|user| {
				let id = user.id;
				let computed = UserComputed {
					permissions: Permissions::default(),
					entitlements: CalculatedEntitlements::new_from_cache(&user.search_index),
					highest_role_rank: -1,
					highest_role_color: None,
					raw_entitlements: None,
				};

				role_ids.extend(computed.entitlements.roles.iter().cloned());

				(id, FullUser { user, computed })
			})
			.collect::<HashMap<_, _>>();

		let roles = global.role_by_id_loader().load_many(role_ids.iter().copied()).await?;

		for user in users.values_mut() {
			user.computed.permissions = compute_permissions(role_order, &roles, &user.computed.entitlements.roles);
			if let Some(active_bans) = user.user.active_bans() {
				user.computed.permissions.merge(active_bans.permissions());
			}

			user.computed.highest_role_rank = compute_highest_role_rank(role_order, &user.computed.entitlements.roles);
			user.computed.highest_role_color =
				compute_highest_role_color(role_order, &roles, &user.computed.entitlements.roles);
		}

		Ok(users)
	}
}

struct UserComputedLoader {
	global: Weak<Global>,
}

impl UserComputedLoader {
	pub fn new(global: Weak<Global>) -> DataLoader<Self> {
		DataLoader::new("UserComputedLoader", Self { global })
	}
}

impl Loader for UserComputedLoader {
	type Error = ();
	type Key = UserId;
	type Value = UserComputed;

	#[tracing::instrument(name = "UserComputedLoader::load", skip(self), fields(key_count = keys.len()))]
	async fn load(&self, keys: Vec<Self::Key>) -> Result<HashMap<Self::Key, Self::Value>, Self::Error> {
		let global = &self.global.upgrade().ok_or(())?;
		let role_order = global.global_config_loader().load(()).await?.ok_or(())?.role_ids;

		let traverse = &EntitlementEdgeGraphTraverse {
			inbound_loader: global.entitlement_edge_inbound_loader(),
			outbound_loader: global.entitlement_edge_outbound_loader(),
		};

		let result = futures::future::try_join_all(keys.into_iter().map(|user_id| async move {
			let raw_entitlements = traverse
				.traversal(
					Direction::Outbound,
					[
						EntitlementEdgeKind::User { user_id },
						EntitlementEdgeKind::GlobalDefaultEntitlementGroup,
					],
				)
				.await?;

			Result::<_, ()>::Ok((user_id, raw_entitlements))
		}))
		.await?;

		let mut role_ids = HashSet::new();

		let mut result = result
			.into_iter()
			.map(|(id, raw_entitlements)| {
				let entitlements = CalculatedEntitlements::new(&raw_entitlements);

				role_ids.extend(entitlements.roles.iter().cloned());

				(
					id,
					UserComputed {
						permissions: Permissions::default(),
						entitlements,
						highest_role_rank: -1,
						highest_role_color: None,
						raw_entitlements: Some(raw_entitlements),
					},
				)
			})
			.collect::<HashMap<_, _>>();

		let roles = global.role_by_id_loader().load_many(role_ids.into_iter()).await?;

		for user in result.values_mut() {
			user.permissions = compute_permissions(&role_order, &roles, &user.entitlements.roles);
			user.highest_role_rank = compute_highest_role_rank(&role_order, &user.entitlements.roles);
			user.highest_role_color = compute_highest_role_color(&role_order, &roles, &user.entitlements.roles);
		}

		Ok(result)
	}
}

fn compute_permissions(role_order: &[RoleId], roles: &HashMap<RoleId, Role>, user_roles: &HashSet<RoleId>) -> Permissions {
	role_order
		.iter()
		.filter(|role_id| user_roles.contains(&role_id))
		.filter_map(|role_id| roles.get(role_id))
		.map(|role| &role.permissions)
		.fold(Permissions::default(), |mut acc, p| {
			acc.merge_ref(p);
			acc
		})
}

fn compute_highest_role_rank(role_order: &[RoleId], user_roles: &HashSet<RoleId>) -> i32 {
	role_order
		.iter()
		.enumerate()
		.rev()
		.find_map(|(idx, role_id)| {
			if user_roles.contains(role_id) {
				Some(idx as i32)
			} else {
				None
			}
		})
		.unwrap_or(-1)
}

fn compute_highest_role_color(
	role_order: &[RoleId],
	roles: &HashMap<RoleId, Role>,
	user_roles: &HashSet<RoleId>,
) -> Option<u32> {
	role_order
		.iter()
		.rev()
		.filter(|role_id| user_roles.contains(role_id))
		.filter_map(|id| roles.get(id))
		.find_map(|role| role.color)
}
