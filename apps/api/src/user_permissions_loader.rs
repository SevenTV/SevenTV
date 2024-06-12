use std::{collections::HashMap, sync::Arc};

use shared::database::{Permissions, User, UserId};

use crate::{global::Global, http::error::ApiError};

pub async fn load_user_permissions(global: &Arc<Global>, user: &User) -> Result<Permissions, ApiError> {
	load_users_permissions(global, [user])
		.await?
		.remove(&user.id)
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)
}

pub async fn load_users_permissions(
	global: &Arc<Global>,
	users: impl IntoIterator<Item = &User> + Clone,
) -> Result<HashMap<UserId, Permissions>, ApiError> {
	let global_config = global
		.global_config_loader()
		.load(())
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	let roles = {
		let mut roles = global
			.role_by_id_loader()
			.load_many(
				users
					.clone()
					.into_iter()
					.flat_map(|user| user.entitled_cache.role_ids.iter().copied()),
			)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		global_config
			.role_ids
			.iter()
			.filter_map(|id| roles.remove(id))
			.collect::<Vec<_>>()
	};

	let mut bans = global
		.active_user_bans_by_user_id_loader()
		.load_many(users.clone().into_iter().map(|u| u.id))
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	let ban_role_ids: Vec<_> = bans.values().flat_map(|b| b.iter().map(|b| b.role_id)).collect();

	let ban_roles = global
		.user_ban_role_by_id_loader()
		.load_many(ban_role_ids)
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

	Ok(users
		.into_iter()
		.map(|u| {
			let mut perms = u.compute_permissions(&roles);

			let user_ban_roles_perms: Permissions = bans
				.remove(&u.id)
				.unwrap_or_default()
				.into_iter()
				.filter_map(|b| ban_roles.get(&b.role_id))
				.map(|r| &r.permissions)
				.collect();

			perms.merge(user_ban_roles_perms);

			(u.id, perms)
		})
		.collect())
}

pub async fn load_user_and_permissions_by_id(
	global: &Arc<Global>,
	user_id: UserId,
) -> Result<Option<(User, Permissions)>, ApiError> {
	let user = match global.user_by_id_loader().load(global, user_id).await {
		Ok(Some(user)) => user,
		Ok(None) => return Ok(None),
		Err(_) => return Err(ApiError::INTERNAL_SERVER_ERROR),
	};

	load_user_permissions(global, &user).await.map(|p| Some((user, p)))
}
