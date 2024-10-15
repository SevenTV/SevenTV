use std::collections::HashMap;
use std::sync::Arc;

use shared::database::emote_set::EmoteSetEmote;
use shared::old_types::{EmotePartialModel, UserPartialModel};

use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;

pub async fn load_emote_set<'a>(
	global: &'a Arc<Global>,
	emote_set_emotes: Vec<EmoteSetEmote>,
	session: &'a Session,
) -> Result<impl Iterator<Item = (EmoteSetEmote, EmotePartialModel)> + 'a, ApiError> {
	let emotes = global
		.emote_by_id_loader
		.load_many_merged(emote_set_emotes.iter().map(|emote| emote.id))
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?;

	let users = global
		.user_loader
		.load_fast_many(global, emotes.emotes.values().map(|emote| emote.owner_id))
		.await
		.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users"))?;

	let cdn_base_url = &global.config.api.cdn_origin;

	let users = users
		.into_values()
		.filter(|user| session.can_view(user))
		.map(|user| {
			// This api doesnt seem to return the user's badges and paints so
			// we can ignore them.
			UserPartialModel::from_db(user, None, None, cdn_base_url)
		})
		.map(|user| (user.id, user))
		.collect::<HashMap<_, _>>();

	Ok(emote_set_emotes.into_iter().filter_map(move |emote_set_emote| {
		let emote = emotes.get(emote_set_emote.id).cloned()?;
		let owner = users.get(&emote.owner_id).cloned();
		let partial = EmotePartialModel::from_db(emote, owner, &global.config.api.cdn_origin);

		Some((emote_set_emote, partial))
	}))
}
