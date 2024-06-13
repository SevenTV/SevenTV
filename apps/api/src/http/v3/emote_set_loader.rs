use std::collections::HashMap;
use std::sync::Arc;

use shared::database::{EmoteSet, EmoteSetEmote, EmoteSetFlags, EmoteSetKind, User, UserId};
use shared::old_types::UserPartialModel;

use super::rest::types::EmotePartialModel;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::user_loader::load_users;

pub async fn load_emote_set(
	global: &Arc<Global>,
	emote_set_emotes: Vec<EmoteSetEmote>,
) -> Result<impl Iterator<Item = (EmoteSetEmote, Option<EmotePartialModel>)>, ApiError> {
	let emotes = global
		.emote_by_id_loader()
		.load_many(emote_set_emotes.iter().map(|emote| emote.emote_id))
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let users = load_users(global, emotes.values().filter_map(|emote| emote.owner_id)).await?;

	let connections = global
		.user_connection_by_user_id_loader()
		.load_many(users.keys().copied())
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let cdn_base_url = &global.config().api.cdn_base_url;

	let users = users
		.into_values()
		.map(|user| {
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

pub fn virtual_user_set(user_id: UserId, display_name: Option<String>, slots: u16) -> EmoteSet {
	let mut name = String::new();
	if let Some(display_name) = display_name {
		name.push_str(&display_name);
		let lower = display_name.to_lowercase();
		if lower.ends_with('s') || lower.ends_with('z') || lower.ends_with('x') {
			name.push_str("' ");
		} else {
			name.push_str("'s ");
		}
	}
	name.push_str("Enabled Emotes");

	EmoteSet {
		id: Default::default(), // set when calling
		owner_id: Some(user_id),
		name,
		kind: EmoteSetKind::Normal,
		tags: Vec::new(),
		flags: EmoteSetFlags::none(),
		capacity: slots as u32,
	}
}

pub async fn get_virtual_set_emotes_for_user(
	global: &Arc<Global>,
	user: &User,
	slots: u16,
) -> Result<Vec<EmoteSetEmote>, ApiError> {
	Ok(global
		.emote_set_emote_by_id_loader()
		.load_many(user.active_emote_set_ids.iter().copied())
		.await
		.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
		.into_values()
		.flatten()
		.take(slots as usize)
		.collect())
}
