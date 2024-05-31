use std::{collections::HashMap, sync::Arc};

use shared::{
	database::{EmoteSet, EmoteSetEmote, EmoteSetFlags, EmoteSetKind, User, UserConnection, UserId},
	old_types::UserPartialModel,
};

use crate::{global::Global, http::error::ApiError};

use super::rest::types::{EmotePartialModel, EmoteSetModel};

pub async fn load_emote_set(
	global: &Arc<Global>,
	emote_set_emotes: Vec<EmoteSetEmote>,
) -> Result<impl Iterator<Item = (EmoteSetEmote, Option<EmotePartialModel>)>, ApiError> {
	let emotes = global
		.emote_by_id_loader()
		.load_many(emote_set_emotes.iter().map(|emote| emote.emote_id))
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let users = global
		.user_by_id_loader()
		.load_many(&global, emotes.values().filter_map(|emote| emote.owner_id))
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let connections = global
		.user_connection_by_user_id_loader()
		.load_many(users.keys().copied())
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let global_config = global
		.global_config_loader()
		.load(())
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

	let roles = {
		let mut roles = global
			.role_by_id_loader()
			.load_many(users.values().flat_map(|user| user.entitled_cache.role_ids.iter().copied()))
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		global_config
			.role_ids
			.iter()
			.filter_map(|id| roles.remove(id))
			.collect::<Vec<_>>()
	};

	let users = users
		.into_iter()
		.map(|(id, user)| {
			let permissions = user.compute_permissions(&roles);
			(id, (user, permissions))
		})
		.collect::<HashMap<_, _>>();

	let cdn_base_url = &global.config().api.cdn_base_url;

	let users = users
		.into_values()
		.map(|(user, _)| {
			// This api doesnt seem to return the user's badges and paints so
			// we can ignore them.
			let connections = connections.get(&user.id).cloned().unwrap_or_default();
			UserPartialModel::from_db(user, connections, None, None, cdn_base_url)
		})
		.map(|user| (user.id, user))
		.collect::<HashMap<_, _>>();

	let emotes = emotes
		.into_iter()
		.filter_map(|(id, emote)| {
			let owner = emote.owner_id.and_then(|id| users.get(&id)).cloned();

			Some((
				id,
				EmotePartialModel::from_db(emote, owner, &global.config().api.cdn_base_url),
			))
		})
		.collect::<HashMap<_, _>>();

	Ok(emote_set_emotes.into_iter().map(move |emote| {
		let partial = emotes.get(&emote.emote_id).cloned();
		(emote, partial)
	}))
}

pub fn fake_user_set(user_id: UserId, slots: u16) -> EmoteSet {
	EmoteSet {
		id: user_id.cast(),
		owner_id: Some(user_id),
		name: "Enabled Emotes".to_string(),
		kind: EmoteSetKind::Normal,
		tags: Vec::new(),
		flags: EmoteSetFlags::none(),
		capacity: slots as u32,
	}
}

pub async fn get_fake_set_for_user_active_sets(
	global: &Arc<Global>,
	user: User,
	user_connections: Vec<UserConnection>,
	slots: u16,
) -> Result<EmoteSetModel, ApiError> {
	let emote_set_emotes: Vec<_> = global
		.emote_set_emote_by_id_loader()
		.load_many(user.active_emote_set_ids.iter().copied())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.into_values()
		.flatten()
		.take(slots as usize)
		.collect();

	Ok(EmoteSetModel::from_db(
		fake_user_set(user.id, slots),
		load_emote_set(global, emote_set_emotes).await?,
		Some(UserPartialModel::from_db(
			user,
			user_connections,
			None,
			None,
			&global.config().api.cdn_base_url,
		)),
	))
}
