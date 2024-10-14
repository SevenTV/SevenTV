use async_graphql::{Enum, SimpleObject};
use shared::database::{emote::EmoteId, emote_set::EmoteSetId, user::UserId};

#[derive(Debug, Clone, SimpleObject)]
pub struct EmoteSet {
	pub id: EmoteSetId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub emotes: Vec<EmoteSetEmote>,
	pub capacity: Option<i32>,
	pub owner_id: Option<UserId>,
	pub kind: EmoteSetKind,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<shared::database::emote_set::EmoteSet> for EmoteSet {
	fn from(value: shared::database::emote_set::EmoteSet) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			emotes: value.emotes.into_iter().map(Into::into).collect(),
			capacity: value.capacity,
			owner_id: value.owner_id,
			kind: value.kind.into(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
pub enum EmoteSetKind {
	Normal,
	Personal,
	Global,
	Special,
}

impl From<shared::database::emote_set::EmoteSetKind> for EmoteSetKind {
	fn from(value: shared::database::emote_set::EmoteSetKind) -> Self {
		match value {
			shared::database::emote_set::EmoteSetKind::Normal => Self::Normal,
			shared::database::emote_set::EmoteSetKind::Personal => Self::Personal,
			shared::database::emote_set::EmoteSetKind::Global => Self::Global,
			shared::database::emote_set::EmoteSetKind::Special => Self::Special,
		}
	}
}

#[derive(Debug, Clone, SimpleObject)]
pub struct EmoteSetEmote {
	pub id: EmoteId,
	pub alias: String,
	pub added_at: chrono::DateTime<chrono::Utc>,
	pub flags: EmoteSetEmoteFlags,
	pub added_by_id: Option<UserId>,
	pub origin_set_id: Option<EmoteSetId>,
}

impl From<shared::database::emote_set::EmoteSetEmote> for EmoteSetEmote {
	fn from(value: shared::database::emote_set::EmoteSetEmote) -> Self {
		Self {
			id: value.id,
			alias: value.alias,
			added_at: value.added_at,
			flags: value.flags.into(),
			added_by_id: value.added_by_id,
			origin_set_id: value.origin_set_id,
		}
	}
}

#[derive(Debug, Clone, SimpleObject)]
pub struct EmoteSetEmoteFlags {
	zero_width: bool,
	override_conflicts: bool,
}

impl From<shared::database::emote_set::EmoteSetEmoteFlags> for EmoteSetEmoteFlags {
	fn from(value: shared::database::emote_set::EmoteSetEmoteFlags) -> Self {
		Self {
			zero_width: value.contains(shared::database::emote_set::EmoteSetEmoteFlags::ZeroWidth),
			override_conflicts: value.contains(shared::database::emote_set::EmoteSetEmoteFlags::OverrideConflicts),
		}
	}
}
