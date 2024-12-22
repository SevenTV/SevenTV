use std::collections::{HashMap, HashSet};
use std::future::IntoFuture;
use std::sync::{Arc, Weak};

use futures::{TryFutureExt, TryStreamExt};
use scuffle_batching::{DataLoader, DataLoaderFetcher};
use shared::database::badge::BadgeId;
use shared::database::emote_set::{EmoteSetId, EmoteSetKind};
use shared::database::entitlement::{
	CalculatedEntitlements, EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy,
};
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::loader::dataloader::BatchLoad;
use shared::database::paint::PaintId;
use shared::database::queries::filter;
use shared::database::role::permissions::{Permissions, PermissionsExt, UserPermission};
use shared::database::role::{Role, RoleId};
use shared::database::user::ban::ActiveBans;
use shared::database::user::{FullUser, User, UserComputed, UserId};
use shared::database::{Id, MongoCollection};
use tracing::Instrument;

use crate::global::Global;

pub struct FullUserLoader {
	pub computed_loader: DataLoader<UserComputedLoader>,
	all_cosmetics_loader: DataLoader<AllCosmeticsLoader>,
}

impl FullUserLoader {
	pub fn new(global: Weak<Global>) -> Self {
		Self {
			computed_loader: UserComputedLoader::new(global.clone()),
			all_cosmetics_loader: AllCosmeticsLoader::new(global.clone()),
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
		user_ids: impl IntoIterator<Item = UserId> + Send,
	) -> Result<HashMap<UserId, FullUser>, ()> {
		let users = global.user_by_id_loader.load_many(user_ids).await?;
		self.load_user_many(global, users.into_values()).await
	}

	/// Performs a full user load fetching all necessary data using the graph
	pub async fn load_user(&self, global: &Arc<Global>, user: User) -> Result<FullUser, ()> {
		let id = user.id;
		self.load_user_many(global, std::iter::once(user))
			.await?
			.remove(&id)
			.ok_or(())
	}

	/// Performs a full user load fetching all necessary data using the graph
	pub async fn load_user_many(
		&self,
		global: &Arc<Global>,
		user: impl IntoIterator<Item = User>,
	) -> Result<HashMap<UserId, FullUser>, ()> {
		let users = user.into_iter().collect::<Vec<_>>();

		let computed = self.computed_loader.load_many(users.iter().map(|user| user.id)).await?;

		let all_cosmetics = if users.iter().any(|user| user.all_cosmetics) {
			self.all_cosmetics_loader.load(()).await?.unwrap_or_default()
		} else {
			AllCosmeticsResult::default()
		};

		let bans = global
			.user_ban_by_user_id_loader
			.load_many(
				users
					.iter()
					.filter_map(|user| if user.has_bans { Some(user.id) } else { None }),
			)
			.await?;

		let profile_pictures = global
			.user_profile_picture_id_loader
			.load_many(users.iter().filter_map(|user| {
				let computed = computed.get(&user.id)?;

				if computed.permissions.has(UserPermission::UseCustomProfilePicture) {
					user.style.active_profile_picture
				} else {
					None
				}
			}))
			.await?;

		Ok(users
			.into_iter()
			.filter_map(|mut user| {
				let mut computed = computed.get(&user.id)?.clone();

				if user.all_cosmetics {
					computed.entitlements.badges.extend(all_cosmetics.badges.iter().copied());
					computed.entitlements.paints.extend(all_cosmetics.paints.iter().copied());
					computed
						.entitlements
						.emote_sets
						.extend(all_cosmetics.emote_sets.iter().copied());

					let entitlements = computed.raw_entitlements.get_or_insert_default();
					for to in all_cosmetics
						.badges
						.iter()
						.copied()
						.map(|id| EntitlementEdgeKind::Badge { badge_id: id })
						.chain(
							all_cosmetics
								.paints
								.iter()
								.copied()
								.map(|id| EntitlementEdgeKind::Paint { paint_id: id }),
						)
						.chain(
							all_cosmetics
								.emote_sets
								.iter()
								.copied()
								.map(|id| EntitlementEdgeKind::EmoteSet { emote_set_id: id }),
						) {
						entitlements.push(EntitlementEdge {
							id: EntitlementEdgeId {
								from: EntitlementEdgeKind::User { user_id: user.id },
								to,
								managed_by: Some(EntitlementEdgeManagedBy::AllCosmetics),
							},
						});
					}
				}

				if let Some(active_bans) = bans.get(&user.id).and_then(|bans| ActiveBans::new(bans)) {
					computed.permissions.merge(active_bans.permissions());
				}

				let active_profile_picture = user
					.style
					.active_profile_picture
					.and_then(|id| profile_pictures.get(&id).cloned());

				user.style.active_badge_id = user.style.active_badge_id.and_then(|id| {
					if computed.permissions.has(UserPermission::UseBadge) && computed.entitlements.badges.contains(&id) {
						Some(id)
					} else {
						None
					}
				});

				user.style.active_paint_id = user.style.active_paint_id.and_then(|id| {
					if computed.permissions.has(UserPermission::UsePaint) && computed.entitlements.paints.contains(&id) {
						Some(id)
					} else {
						None
					}
				});

				Some((
					user.id,
					FullUser {
						user,
						computed,
						active_profile_picture,
					},
				))
			})
			.collect())
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast(&self, global: &Arc<Global>, user_id: UserId) -> Result<Option<FullUser>, ()> {
		self.load_fast_many(global, std::iter::once(user_id))
			.await
			.map(|mut users| users.remove(&user_id))
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast_many(
		&self,
		global: &Arc<Global>,
		user_ids: impl IntoIterator<Item = UserId> + Send,
	) -> Result<HashMap<UserId, FullUser>, ()> {
		let users = global.user_by_id_loader.load_many(user_ids).await?;
		self.load_fast_user_many(global, users.into_values()).await
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast_user(&self, global: &Arc<Global>, user: User) -> Result<FullUser, ()> {
		let id = user.id;
		self.load_fast_user_many(global, std::iter::once(user))
			.await?
			.remove(&id)
			.ok_or(())
	}

	/// Performs a fast user load fetching using the cache'ed data
	pub async fn load_fast_user_many(
		&self,
		global: &Arc<Global>,
		user: impl IntoIterator<Item = User>,
	) -> Result<HashMap<UserId, FullUser>, ()> {
		let mut role_ids = HashSet::new();

		let mut users = user
			.into_iter()
			.map(|user| {
				let id = user.id;
				let computed = UserComputed {
					permissions: Permissions::default(),
					entitlements: CalculatedEntitlements::new(user.cached.entitlements.iter().cloned()),
					highest_role_rank: -1,
					highest_role_color: None,
					raw_entitlements: None,
					roles: vec![],
				};

				role_ids.extend(computed.entitlements.roles.iter().cloned());

				(
					id,
					FullUser {
						user,
						computed,
						active_profile_picture: None,
					},
				)
			})
			.collect::<HashMap<_, _>>();

		let all_cosmetics = if users.values().any(|user| user.all_cosmetics) {
			self.all_cosmetics_loader.load(()).await?.unwrap_or_default()
		} else {
			AllCosmeticsResult::default()
		};

		let mut roles: Vec<_> = global
			.role_by_id_loader
			.load_many(role_ids.iter().copied())
			.await?
			.into_values()
			.collect();

		let bans = global
			.user_ban_by_user_id_loader
			.load_many(
				users
					.values()
					.filter_map(|user| if user.has_bans { Some(user.id) } else { None }),
			)
			.await?;

		roles.sort_by_key(|r| r.rank);

		for user in users.values_mut() {
			user.computed.permissions = compute_permissions(&roles, &user.computed.entitlements.roles);
			if let Some(active_bans) = bans.get(&user.id).and_then(|bans| ActiveBans::new(bans)) {
				user.computed.permissions.merge(active_bans.permissions());
			}
		}

		let profile_pictures = global
			.user_profile_picture_id_loader
			.load_many(users.values().filter_map(|user| {
				if user.computed.permissions.has(UserPermission::UseCustomProfilePicture) {
					user.style.active_profile_picture
				} else {
					None
				}
			}))
			.await?;

		for user in users.values_mut() {
			if user.all_cosmetics {
				user.computed.entitlements.badges.extend(all_cosmetics.badges.iter().copied());
				user.computed.entitlements.paints.extend(all_cosmetics.paints.iter().copied());
				user.computed
					.entitlements
					.emote_sets
					.extend(all_cosmetics.emote_sets.iter().copied());

				let user_id = user.id;

				let entitlements = user.computed.raw_entitlements.get_or_insert_default();
				for to in all_cosmetics
					.badges
					.iter()
					.copied()
					.map(|id| EntitlementEdgeKind::Badge { badge_id: id })
					.chain(
						all_cosmetics
							.paints
							.iter()
							.copied()
							.map(|id| EntitlementEdgeKind::Paint { paint_id: id }),
					)
					.chain(
						all_cosmetics
							.emote_sets
							.iter()
							.copied()
							.map(|id| EntitlementEdgeKind::EmoteSet { emote_set_id: id }),
					) {
					entitlements.push(EntitlementEdge {
						id: EntitlementEdgeId {
							from: EntitlementEdgeKind::User { user_id },
							to,
							managed_by: Some(EntitlementEdgeManagedBy::AllCosmetics),
						},
					});
				}
			}

			user.computed.highest_role_rank = compute_highest_role_rank(&roles, &user.computed.entitlements.roles);
			user.computed.highest_role_color = compute_highest_role_color(&roles, &user.computed.entitlements.roles);
			user.computed.roles = roles
				.iter()
				.map(|r| r.id)
				.filter(|r| user.computed.entitlements.roles.contains(r))
				.collect();

			user.active_profile_picture = user
				.style
				.active_profile_picture
				.and_then(|id| profile_pictures.get(&id).cloned());

			user.user.style.active_badge_id = user.style.active_badge_id.and_then(|id| {
				if user.computed.permissions.has(UserPermission::UseBadge) && user.computed.entitlements.badges.contains(&id)
				{
					Some(id)
				} else {
					None
				}
			});

			user.user.style.active_paint_id = user.style.active_paint_id.and_then(|id| {
				if user.computed.permissions.has(UserPermission::UsePaint) && user.computed.entitlements.paints.contains(&id)
				{
					Some(id)
				} else {
					None
				}
			});
		}

		Ok(users)
	}
}

pub struct UserComputedLoader {
	global: Weak<Global>,
	name: String,
}

impl UserComputedLoader {
	pub fn new(global: Weak<Global>) -> DataLoader<Self> {
		Self::new_with_config(
			global,
			"UserComputedLoader".to_string(),
			1000,
			500,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		global: Weak<Global>,
		name: String,
		batch_size: usize,
		concurrency: usize,
		sleep_duration: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { global, name }, batch_size, concurrency, sleep_duration)
	}
}

impl DataLoaderFetcher for UserComputedLoader {
	type Key = UserId;
	type Value = UserComputed;

	async fn load(
		&self,
		keys: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		let _batch = BatchLoad::new(&self.name, keys.len());

		let global = &self.global.upgrade()?;

		let traverse = &EntitlementEdgeGraphTraverse {
			inbound_loader: &global.entitlement_edge_inbound_loader,
			outbound_loader: &global.entitlement_edge_outbound_loader,
		};

		let result = futures::future::try_join_all(keys.into_iter().map(|user_id| async move {
			let span = tracing::info_span!("traversal", user_id = %user_id);
			let raw_entitlements = traverse
				.traversal(
					Direction::Outbound,
					std::iter::once(EntitlementEdgeKind::GlobalDefaultEntitlementGroup)
						.chain((!user_id.is_nil()).then_some(EntitlementEdgeKind::User { user_id })),
				)
				.instrument(span)
				.await?;

			Result::<_, ()>::Ok((user_id, raw_entitlements))
		}))
		.await
		.ok()?;

		let mut role_ids = HashSet::new();

		let mut result = result
			.into_iter()
			.map(|(id, raw_entitlements)| {
				let entitlements = CalculatedEntitlements::new(raw_entitlements.iter().map(|e| e.id.to.clone()));

				role_ids.extend(entitlements.roles.iter().cloned());

				(
					id,
					UserComputed {
						permissions: Permissions::default(),
						entitlements,
						highest_role_rank: -1,
						highest_role_color: None,
						roles: vec![],
						raw_entitlements: Some(raw_entitlements),
					},
				)
			})
			.collect::<HashMap<_, _>>();

		let mut roles: Vec<_> = global
			.role_by_id_loader
			.load_many(role_ids.into_iter())
			.await
			.ok()?
			.into_values()
			.collect();
		roles.sort_by_key(|r| r.rank);

		for user in result.values_mut() {
			user.permissions = compute_permissions(&roles, &user.entitlements.roles);
			user.highest_role_rank = compute_highest_role_rank(&roles, &user.entitlements.roles);
			user.highest_role_color = compute_highest_role_color(&roles, &user.entitlements.roles);
			user.roles = roles
				.iter()
				.map(|r| r.id)
				.filter(|r| user.entitlements.roles.contains(r))
				.collect();
		}

		Some(result)
	}
}

struct AllCosmeticsLoader {
	global: Weak<Global>,
	name: String,
}

impl AllCosmeticsLoader {
	pub fn new(global: Weak<Global>) -> DataLoader<Self> {
		Self::new_with_config(
			global,
			"AllCosmeticsLoader".to_string(),
			1,
			5,
			std::time::Duration::from_millis(5),
		)
	}

	pub fn new_with_config(
		global: Weak<Global>,
		name: String,
		batch_size: usize,
		concurrency: usize,
		sleep_duration: std::time::Duration,
	) -> DataLoader<Self> {
		DataLoader::new(Self { global, name }, batch_size, concurrency, sleep_duration)
	}
}

#[derive(Debug, Clone, Default)]
struct AllCosmeticsResult {
	badges: Vec<BadgeId>,
	paints: Vec<PaintId>,
	emote_sets: Vec<EmoteSetId>,
}

impl DataLoaderFetcher for AllCosmeticsLoader {
	type Key = ();
	type Value = AllCosmeticsResult;

	async fn load(
		&self,
		_: std::collections::HashSet<Self::Key>,
	) -> Option<std::collections::HashMap<Self::Key, Self::Value>> {
		use shared::database::badge::Badge;
		use shared::database::emote_set::EmoteSet;
		use shared::database::paint::Paint;

		let _batch = BatchLoad::new(&self.name, 1);
		let global = &self.global.upgrade()?;

		async fn load_cosmetics<T: MongoCollection + serde::de::DeserializeOwned>(
			global: &Global,
			filter: impl Into<filter::Filter<T>>,
		) -> Vec<Id<T>> {
			#[derive(Debug, Clone, serde::Deserialize)]
			struct Ret<I> {
				#[serde(rename = "_id")]
				id: Id<I>,
			}

			let filter = filter.into();
			let results: Vec<Ret<T>> = match global
				.db
				.collection(T::COLLECTION_NAME)
				.find(filter.to_document())
				.projection(bson::doc! {
					"_id": 1,
				})
				.into_future()
				.and_then(|f| f.try_collect())
				.await
			{
				Ok(results) => results,
				Err(err) => {
					tracing::error!("failed to load all cosmetics: {err}");
					vec![]
				}
			};

			results.into_iter().map(|r| r.id).collect()
		}

		let badges = load_cosmetics::<Badge>(
			global,
			filter::filter! {
				Badge {}
			},
		)
		.await;

		let paints = load_cosmetics::<Paint>(
			global,
			filter::filter! {
				Paint {}
			},
		)
		.await;

		let emote_sets = load_cosmetics::<EmoteSet>(
			global,
			filter::filter! {
				EmoteSet {
					#[query(serde)]
					kind: EmoteSetKind::Special,
				}
			},
		)
		.await;

		let result = AllCosmeticsResult {
			badges,
			paints,
			emote_sets,
		};

		Some([((), result)].into_iter().collect())
	}
}

fn compute_permissions(sorted_roles: &[Role], user_roles: &HashSet<RoleId>) -> Permissions {
	sorted_roles
		.iter()
		.filter(|role| user_roles.contains(&role.id))
		.map(|role| &role.permissions)
		.fold(Permissions::default(), |mut acc, p| {
			acc.merge_ref(p);
			acc
		})
}

fn compute_highest_role_rank(sorted_roles: &[Role], user_roles: &HashSet<RoleId>) -> i32 {
	sorted_roles
		.iter()
		.rev()
		.find_map(|role| {
			if user_roles.contains(&role.id) {
				Some(role.rank)
			} else {
				None
			}
		})
		.unwrap_or(-1)
}

fn compute_highest_role_color(sorted_roles: &[Role], user_roles: &HashSet<RoleId>) -> Option<i32> {
	sorted_roles
		.iter()
		.rev()
		.filter(|role| user_roles.contains(&role.id))
		.find_map(|role| role.color)
}
