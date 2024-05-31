use shared::{database::{EmoteId, EmoteSet, EmoteSetEmote, EmoteSetEmoteFlag, EmoteSetFlags, EmoteSetId, UserId}, old_types::{ActiveEmoteFlagModel, EmoteSetFlagModel, UserPartialModel}};

use super::{is_default, EmotePartialModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L9
pub struct EmoteSetModel {
	pub id: EmoteSetId,
	pub name: String,
	pub flags: EmoteSetFlagModel,
	pub tags: Vec<String>,
	pub immutable: bool,
	pub privileged: bool,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub emotes: Vec<ActiveEmoteModel>,
	#[serde(skip_serializing_if = "is_default")]
	pub emote_count: i32,
	pub capacity: i32,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub origins: Vec<EmoteSetOrigin>,
	pub owner: Option<UserPartialModel>,
}

impl EmoteSetModel {
	pub fn from_db(
		value: EmoteSet,
		emotes: impl IntoIterator<Item = (EmoteSetEmote, Option<EmotePartialModel>)>,
		owner: Option<UserPartialModel>,
	) -> Self {
		let emotes = emotes
			.into_iter()
			.map(|(emote, data)| ActiveEmoteModel::from_db(emote, data))
			.collect::<Vec<_>>();

		Self {
			flags: EmoteSetFlagModel::from_db(&value),
			id: value.id,
			name: value.name,
			tags: value.tags,
			immutable: value.flags.contains(EmoteSetFlags::Immutable),
			privileged: value.flags.contains(EmoteSetFlags::Privileged),
			emote_count: emotes.len() as i32,
			capacity: value.capacity as i32,
			emotes,
			origins: Vec::new(),
			owner,
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L23
pub struct EmoteSetPartialModel {
	pub id: EmoteSetId,
	pub name: String,
	pub flags: EmoteSetFlagModel,
	pub tags: Vec<String>,
	pub capacity: i32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner: Option<UserPartialModel>,
}

impl EmoteSetPartialModel {
	pub fn from_db(value: EmoteSet, owner: Option<UserPartialModel>) -> Self {
		EmoteSetPartialModel {
			flags: EmoteSetFlagModel::from_db(&value),
			id: value.id,
			name: value.name,
			capacity: value.capacity as i32,
			tags: value.tags,
			owner,
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L45
pub struct ActiveEmoteModel {
	pub id: EmoteId,
	pub name: String,
	pub flags: ActiveEmoteFlagModel,
	pub timestamp: i64,
	pub actor_id: Option<UserId>,
	pub data: Option<EmotePartialModel>,
	pub origin_id: Option<EmoteSetId>,
}

impl ActiveEmoteModel {
	pub fn from_db(value: EmoteSetEmote, data: Option<EmotePartialModel>) -> Self {
		ActiveEmoteModel {
			id: value.emote_id,
			actor_id: value.added_by_id,
			name: value.name,
			timestamp: value.id.timestamp_ms() as i64,
			origin_id: None,
			flags: {
				let mut flags = ActiveEmoteFlagModel::none();

				if value.flags.contains(EmoteSetEmoteFlag::ZeroWidth) {
					flags |= ActiveEmoteFlagModel::ZeroWidth;
				}

				if value.flags.contains(EmoteSetEmoteFlag::OverrideConflicts) {
					flags |= ActiveEmoteFlagModel::OverrideBetterTTV
						| ActiveEmoteFlagModel::OverrideTwitchGlobal
						| ActiveEmoteFlagModel::OverrideTwitchSubscriber;
				}

				flags
			},
			data,
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/emote-set.model.go#L64
pub struct EmoteSetOrigin {
	pub id: EmoteSetId,
	pub weight: i32,
	pub slices: Vec<u32>,
}
