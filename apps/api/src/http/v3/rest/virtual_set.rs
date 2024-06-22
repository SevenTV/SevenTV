use std::sync::Arc;

use shared::database::{User, UserConnection};
use shared::old_types::UserPartialModel;

use super::types::EmoteSetModel;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::emote_set_loader::{get_virtual_set_emotes_for_user, load_emote_set, virtual_user_set};

pub async fn get_virtual_rest_set_for_user(
	global: &Arc<Global>,
	user: User,
	user_connections: Vec<UserConnection>,
	slots: u16,
) -> Result<EmoteSetModel, ApiError> {
	let emote_set_emotes: Vec<_> = get_virtual_set_emotes_for_user(global, &user, slots).await?;

	let display_name = user_connections
		.iter()
		.find(|conn| conn.main_connection)
		.map(|c| c.platform_display_name.clone());

	Ok(EmoteSetModel::from_db(
		virtual_user_set(user.id, display_name, slots),
		load_emote_set(global, emote_set_emotes).await?,
		Some(UserPartialModel::from_db(
			user,
			user_connections,
			None,
			None,
			&global.config().api.cdn_origin,
		)),
	))
}
