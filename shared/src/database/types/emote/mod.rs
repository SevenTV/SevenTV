mod attribution;
mod set;

use std::sync::Arc;

pub use attribution::*;
pub use set::*;
use crate::types::old::{
	EmoteLifecycleModel, EmoteModel, EmotePartialModel, EmoteVersionModel, EmoteVersionState, ImageHost, ImageHostKind,
	UserPartialModel,
};

use super::FileSet;
use crate::database::Table;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Emote {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub default_name: String,
	pub tags: Vec<String>,
	pub animated: bool,
	pub file_set_id: ulid::Ulid,
	pub settings: EmoteSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Emote {
	pub fn into_old_model(self, owner: Option<UserPartialModel>, file_set: &FileSet, cdn_base_url: &str) -> EmoteModel {
		let partial = self.into_old_model_partial(owner, file_set, cdn_base_url);

		EmoteModel {
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
				lifecycle: EmoteLifecycleModel::Live,
				state: partial.state,
				listed: partial.listed,
				animated: partial.animated,
				host: Some(partial.host),
				created_at: partial.id.timestamp_ms() as i64,
			}],
		}
	}

	pub fn into_old_model_partial(
		self,
		owner: Option<UserPartialModel>,
		file_set: &FileSet,
		cdn_base_url: &str,
	) -> EmotePartialModel {
		EmotePartialModel {
			id: self.id,
			name: self.default_name,
			animated: self.animated,
			tags: self.tags,
			owner,
			flags: {
				let mut flags = crate::types::old::EmoteFlagsModel::none();

				if self.settings.private {
					flags |= crate::types::old::EmoteFlagsModel::Private;
				}

				if self.settings.default_zero_width {
					flags |= crate::types::old::EmoteFlagsModel::ZeroWidth;
				}

				if self.settings.nsfw {
					flags |= crate::types::old::EmoteFlagsModel::Sexual;
				}

				flags
			},
			state: {
				let mut state = Vec::new();

				if let Some(approved_personal) = self.settings.approved_personal {
					if approved_personal {
						state.push(crate::types::old::EmoteVersionState::AllowPersonal);
					} else {
						state.push(crate::types::old::EmoteVersionState::NoPersonal);
					}
				}

				if self.settings.public_listed {
					state.push(crate::types::old::EmoteVersionState::Listed);
				}

				state
			},
			lifecycle: if file_set.properties.pending() {
				crate::types::old::EmoteLifecycleModel::Pending
			} else {
				crate::types::old::EmoteLifecycleModel::Live
			},
			listed: self.settings.public_listed,
			host: ImageHost::new(
				cdn_base_url,
				ImageHostKind::Emote,
				self.id,
				file_set.properties.as_old_image_files(),
			),
		}
	}
}

impl Table for Emote {
	const TABLE_NAME: &'static str = "emotes";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct EmoteSettings {
	pub public_listed: bool,
	pub private: bool,
	pub nsfw: bool,
	pub default_zero_width: bool,
	pub approved_personal: Option<bool>,
}
