use shared::database::emote::{Emote, EmoteId};
use shared::old_types::image::ImageHost;
use shared::old_types::{
	EmoteFlagsModel, EmoteLifecycleModel, EmotePartialModel, EmoteVersionModel, EmoteVersionState, UserPartialModel,
};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default, deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L12
pub struct EmoteModel {
	pub id: EmoteId,
	pub name: String,
	pub flags: EmoteFlagsModel,
	pub tags: Vec<String>,
	pub lifecycle: EmoteLifecycleModel,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	pub owner: Option<UserPartialModel>,
	pub host: ImageHost,
	pub versions: Vec<EmoteVersionModel>,
}

impl EmoteModel {
	pub fn from_db(value: Emote, owner: Option<UserPartialModel>, cdn_base_url: &url::Url) -> Self {
		let partial = EmotePartialModel::from_db(value, owner, cdn_base_url);

		Self {
			id: partial.id,
			name: partial.name.clone(),
			flags: partial.flags,
			tags: partial.tags,
			lifecycle: partial.lifecycle,
			state: partial.state.clone(),
			listed: partial.listed,
			animated: partial.animated,
			owner: partial.owner,
			host: partial.host.clone(),
			versions: vec![EmoteVersionModel {
				id: partial.id,
				name: partial.name,
				description: String::new(),
				lifecycle: partial.lifecycle,
				state: partial.state,
				listed: partial.listed,
				animated: partial.animated,
				host: Some(partial.host),
				created_at: partial.id.timestamp_ms(),
			}],
		}
	}
}
