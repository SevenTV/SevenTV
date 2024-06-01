use shared::database::EmoteSetId;
use shared::old_types::{UserConnectionPartialModel, UserConnectionPlatformModel};

use super::{EmoteSetModel, UserModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user-connection.model.go#L10
pub struct UserConnectionModel {
	pub id: String,
	pub platform: UserConnectionPlatformModel,
	pub username: String,
	pub display_name: String,
	pub linked_at: u64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<EmoteSetId>,
	pub emote_set: Option<EmoteSetModel>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<UserModel>,
}

impl From<UserConnectionPartialModel> for UserConnectionModel {
	fn from(value: UserConnectionPartialModel) -> Self {
		Self {
			id: value.id,
			platform: value.platform,
			username: value.username,
			display_name: value.display_name,
			linked_at: value.linked_at,
			emote_capacity: value.emote_capacity,
			emote_set_id: value.emote_set_id,
			emote_set: None,
			user: None,
		}
	}
}
