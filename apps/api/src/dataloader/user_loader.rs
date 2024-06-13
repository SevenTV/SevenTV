use std::collections::HashMap;
use std::sync::Arc;

use shared::database::{Permissions, User, UserBan, UserId};

use crate::global::Global;
use crate::http::error::ApiError;

// Loading users is quite complex.
// Loading a user always requires loading the user itself and their bans to see
// if they are black holed. Loading user permissions requires loading the user,
// their bans, and their roles.

/// Internal helper function to load
/// - users
/// - active bans
async fn load_users_and_bans(
	global: &Arc<Global>,
	user_ids: impl IntoIterator<Item = UserId>,
) -> Result<HashMap<UserId, (User, Vec<UserBan>)>, ApiError> {
	let users = global
		.user_by_id_loader()
		.load_many(user_ids)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	let mut active_bans = global
		.active_user_bans_by_user_id_loader()
		.load_many(users.keys().copied())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	let ban_role_ids: Vec<_> = active_bans.values().flat_map(|b| b.iter().map(|b| b.role_id)).collect();

	let ban_roles = global
		.user_ban_role_by_id_loader()
		.load_many(ban_role_ids)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	Ok(users
		.into_iter()
		.filter_map(|(id, user)| {
			let user_bans = active_bans.remove(&id).unwrap_or_default();
			// keep this user if all bans are not black holes (if there aren't any black
			// holes)
			user_bans
				.iter()
				.all(|b| ban_roles.get(&b.role_id).map(|r| !r.black_hole).unwrap_or(true))
				.then(|| (id, (user, user_bans)))
		})
		.collect())
}

/// Internal helper function to load
/// - users
/// - active bans
/// - user permissions
async fn load_users_and_bans_and_permissions(
	global: &Arc<Global>,
	user_ids: impl IntoIterator<Item = UserId>,
) -> Result<HashMap<UserId, (User, Vec<UserBan>, Permissions)>, ApiError> {
	let users = global
		.user_by_id_loader()
		.load_many(user_ids)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	let global_config = global
		.global_config_loader()
		.load(())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	let roles = {
		let mut roles = global
			.role_by_id_loader()
			.load_many(users.values().flat_map(|user| user.grants.role_ids.iter().copied()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		global_config
			.role_ids
			.iter()
			.filter_map(|id| roles.remove(id))
			.collect::<Vec<_>>()
	};

	let mut active_bans = global
		.active_user_bans_by_user_id_loader()
		.load_many(users.keys().copied())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	let ban_role_ids: Vec<_> = active_bans.values().flat_map(|b| b.iter().map(|b| b.role_id)).collect();

	let ban_roles = global
		.user_ban_role_by_id_loader()
		.load_many(ban_role_ids)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	Ok(users
		.into_iter()
		.filter_map(|(id, user)| {
			let user_bans = active_bans.remove(&id).unwrap_or_default();
			// keep this user if all bans are not black holes (if there aren't any black
			// holes)
			user_bans
				.iter()
				.all(|b| ban_roles.get(&b.role_id).map(|r| !r.black_hole).unwrap_or(true))
				.then(|| {
					let mut perms = user.compute_permissions(&roles);

					let user_ban_roles_perms: Permissions = active_bans
						.remove(&id)
						.unwrap_or_default()
						.into_iter()
						.filter_map(|b| ban_roles.get(&b.role_id))
						.map(|r| &r.permissions)
						.collect();

					perms.merge(user_ban_roles_perms);

					(id, (user, user_bans, perms))
				})
		})
		.collect())
}

/// Load users by ids
pub async fn load_users(
	global: &Arc<Global>,
	user_ids: impl IntoIterator<Item = UserId>,
) -> Result<HashMap<UserId, User>, ApiError> {
	Ok(load_users_and_bans(global, user_ids)
		.await?
		.into_iter()
		.map(|(id, (user, _))| (id, user))
		.collect())
}

/// Load one user by user id
pub async fn load_user(global: &Arc<Global>, user_id: UserId) -> Result<Option<User>, ApiError> {
	Ok(load_users(global, [user_id]).await?.remove(&user_id))
}

pub async fn load_users_and_permissions(
	global: &Arc<Global>,
	user_ids: impl IntoIterator<Item = UserId>,
) -> Result<HashMap<UserId, (User, Permissions)>, ApiError> {
	Ok(load_users_and_bans_and_permissions(global, user_ids)
		.await?
		.into_iter()
		.map(|(id, (user, _, perms))| (id, (user, perms)))
		.collect())
}

/// Load one user and their permissions by user id
pub async fn load_user_and_permissions(
	global: &Arc<Global>,
	user_id: UserId,
) -> Result<Option<(User, Permissions)>, ApiError> {
	Ok(load_users_and_permissions(global, [user_id]).await?.remove(&user_id))
}
