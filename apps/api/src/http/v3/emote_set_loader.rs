use std::collections::HashMap;
use std::sync::Arc;

use shared::database::emote_set::EmoteSetEmote;
use shared::database::role::permissions::{FlagPermission, PermissionsExt};
use shared::database::user::UserId;
use shared::old_types::UserPartialModel;

use super::rest::types::EmotePartialModel;
use crate::global::Global;
use crate::http::error::ApiError;

pub async fn load_emote_set(
	global: &Arc<Global>,
	emote_set_emotes: Vec<EmoteSetEmote>,
	actor_id: Option<UserId>,
	view_hidden: bool,
) -> Result<impl Iterator<Item = (EmoteSetEmote, Option<EmotePartialModel>)>, ApiError> {
	let emotes = global
		.emote_by_id_loader()
		.load_many(emote_set_emotes.iter().map(|emote| emote.id))
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let users = global
		.user_loader()
		.load_fast_many(global, emotes.values().map(|emote| emote.owner_id))
		.await
		.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

	let cdn_base_url = &global.config().api.cdn_origin;

	let users = users
		.into_values()
		.filter(|user| Some(user.id) == actor_id || !user.has(FlagPermission::Hidden) || view_hidden)
		.map(|user| {
			// This api doesnt seem to return the user's badges and paints so
			// we can ignore them.
			UserPartialModel::from_db(user, None, None, cdn_base_url)
		})
		.map(|user| (user.id, user))
		.collect::<HashMap<_, _>>();

	let emotes = emotes
		.into_iter()
		.filter_map(|(id, emote)| {
			let owner = users.get(&emote.owner_id).cloned();

			Some((id, EmotePartialModel::from_db(emote, owner, &global.config().api.cdn_origin)))
		})
		.collect::<HashMap<_, _>>();

	Ok(emote_set_emotes.into_iter().map(move |emote| {
		let partial = emotes.get(&emote.id).cloned();
		(emote, partial)
	}))
}
