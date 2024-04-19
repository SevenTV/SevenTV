use std::sync::Arc;

use bitmask_enum::bitmask;

use super::{FileSet, FileSetId, UserId};
use crate::database::{Collection, Id};
use crate::types::old::{
	EmoteLifecycleModel, EmoteModel, EmotePartialModel, EmoteVersionModel, EmoteVersionState, ImageHost, ImageHostKind,
	UserPartialModel,
};

pub type EmoteId = Id<Emote>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Emote {
	#[serde(rename = "_id", with = "crate::database::id::bson")]
	pub id: EmoteId,
	pub owner_id: Option<UserId>,
	pub default_name: String,
	pub tags: Vec<String>,
	pub animated: bool,
	pub file_set_id: FileSetId,
	pub flags: EmoteFlags,
	pub attribution: Vec<EmoteAttribution>,
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
				created_at: partial.id.timestamp_ms(),
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

				if self.flags.contains(EmoteFlags::Private) {
					flags |= crate::types::old::EmoteFlagsModel::Private;
				}

				if self.flags.contains(EmoteFlags::DefaultZeroWidth) {
					flags |= crate::types::old::EmoteFlagsModel::ZeroWidth;
				}

				if self.flags.contains(EmoteFlags::Nsfw) {
					flags |= crate::types::old::EmoteFlagsModel::Sexual;
				}

				flags
			},
			state: {
				let mut state = Vec::new();

				if self.flags.contains(EmoteFlags::ApprovedPersonal) && !self.flags.contains(EmoteFlags::DeniedPersonal) {
					state.push(crate::types::old::EmoteVersionState::AllowPersonal);
				} else if self.flags.contains(EmoteFlags::DeniedPersonal) {
					state.push(crate::types::old::EmoteVersionState::NoPersonal);
				}

				if self.flags.contains(EmoteFlags::PublicListed) {
					state.push(crate::types::old::EmoteVersionState::Listed);
				}

				state
			},
			lifecycle: if file_set.properties.pending() {
				crate::types::old::EmoteLifecycleModel::Pending
			} else {
				crate::types::old::EmoteLifecycleModel::Live
			},
			listed: self.flags.contains(EmoteFlags::PublicListed),
			host: ImageHost::new(
				cdn_base_url,
				ImageHostKind::Emote,
				self.id.cast(),
				file_set.properties.as_old_image_files(),
			),
		}
	}
}

impl Collection for Emote {
	const COLLECTION_NAME: &'static str = "emotes";
}

#[bitmask(u8)]
pub enum EmoteFlags {
	PublicListed = 1 << 0,
	Private = 1 << 1,
	Nsfw = 1 << 2,
	DefaultZeroWidth = 1 << 3,
	ApprovedPersonal = 1 << 4,
	DeniedPersonal = 1 << 5,
}

impl Default for EmoteFlags {
	fn default() -> Self {
		Self::none()
	}
}

impl serde::Serialize for EmoteFlags {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for EmoteFlags {
	fn deserialize<D>(deserializer: D) -> Result<EmoteFlags, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = u8::deserialize(deserializer)?;
		Ok(EmoteFlags::from(bits))
	}
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmoteAttribution {
	pub user_id: UserId,
	pub added_at: chrono::DateTime<chrono::Utc>,
}
