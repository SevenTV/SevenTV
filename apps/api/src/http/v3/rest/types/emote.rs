use shared::database::{Emote, EmoteFlags, EmoteId, UserId};
use shared::old_types::{EmoteFlagsModel, ImageHost, ImageHostKind, UserPartialModel};

use crate::http::v3::types::{EmoteLifecycleModel, EmoteVersionState};

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
	#[serde(skip)]
	pub owner_id: UserId,
	pub host: ImageHost,
	pub versions: Vec<EmoteVersionModel>,
}

impl EmoteModel {
	pub fn from_db(value: Emote, owner: Option<UserPartialModel>, cdn_base_url: &str) -> Self {
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
			owner_id: partial.owner.as_ref().map(|u| u.id).unwrap_or_default(),
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L27
pub struct EmotePartialModel {
	pub id: EmoteId,
	pub name: String,
	pub flags: EmoteFlagsModel,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<String>,
	pub lifecycle: EmoteLifecycleModel,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserPartialModel>,
	pub host: ImageHost,
}

impl EmotePartialModel {
	pub fn from_db(value: Emote, owner: Option<UserPartialModel>, cdn_base_url: &str) -> Self {
		Self {
			id: value.id,
			name: value.default_name,
			animated: value.animated,
			tags: value.tags,
			owner,
			state: EmoteVersionState::from_db(&value.flags),
			flags: value.flags.into(),
			lifecycle: if value.merged_into.is_some() {
				EmoteLifecycleModel::Deleted
			} else if value.image_set.input.is_pending() {
				EmoteLifecycleModel::Pending
			} else {
				EmoteLifecycleModel::Live
			},
			listed: value.flags.contains(EmoteFlags::PublicListed),
			host: ImageHost::from_image_set(&value.image_set, cdn_base_url, ImageHostKind::Emote, &value.id),
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote.model.go#L40
pub struct EmoteVersionModel {
	pub id: EmoteId,
	pub name: String,
	pub description: String,
	pub lifecycle: EmoteLifecycleModel,
	pub state: Vec<EmoteVersionState>,
	pub listed: bool,
	pub animated: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub host: Option<ImageHost>,
	#[serde(rename = "createdAt")]
	pub created_at: u64,
}
