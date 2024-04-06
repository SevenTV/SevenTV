mod attribution;
mod set;

use std::sync::Arc;

pub use attribution::*;
pub use set::*;
use shared::types::old::{ImageHost, ImageHostKind, UserPartialModel};

use super::FileSet;
use crate::database::Table;
use crate::global::Global;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Emote {
	pub id: ulid::Ulid,
	pub owner_id: Option<ulid::Ulid>,
	pub default_name: String,
	pub tags: Vec<String>,
	pub animated: bool,
	pub file_set_id: ulid::Ulid,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub settings: EmoteSettings,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Emote {
	pub fn into_old_model_partial(
		self,
		global: &Arc<Global>,
		owner: Option<UserPartialModel>,
		file_set: &FileSet,
	) -> shared::types::old::EmotePartialModel {
		shared::types::old::EmotePartialModel {
			id: self.id,
			name: self.default_name,
			animated: self.animated,
			tags: self.tags,
			owner,
			flags: {
				let mut flags = shared::types::old::EmoteFlagsModel::none();

				if self.settings.private {
					flags |= shared::types::old::EmoteFlagsModel::Private;
				}

				if self.settings.default_zero_width {
					flags |= shared::types::old::EmoteFlagsModel::ZeroWidth;
				}

				if self.settings.nsfw {
					flags |= shared::types::old::EmoteFlagsModel::Sexual;
				}

				flags
			},
			state: {
				let mut state = Vec::new();

				if let Some(approved_personal) = self.settings.approved_personal {
					if approved_personal {
						state.push(shared::types::old::EmoteVersionState::AllowPersonal);
					} else {
						state.push(shared::types::old::EmoteVersionState::NoPersonal);
					}
				}

				if self.settings.public_listed {
					state.push(shared::types::old::EmoteVersionState::Listed);
				}

				state
			},
			lifecycle: if file_set.properties.pending() {
				shared::types::old::EmoteLifecycleModel::Pending
			} else {
				shared::types::old::EmoteLifecycleModel::Live
			},
			listed: self.settings.public_listed,
			host: ImageHost::new(
				&global.config().api.cdn_base_url,
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
