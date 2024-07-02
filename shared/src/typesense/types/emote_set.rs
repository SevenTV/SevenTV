use chrono::Utc;

use super::{TypesenseCollection, TypesenseGenericCollection};
use crate::database;
use crate::database::emote::EmoteId;
use crate::database::emote_set::{EmoteSetId, EmoteSetKind};
use crate::database::user::UserId;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "emote_sets")]
#[serde(deny_unknown_fields)]
pub struct EmoteSet {
	pub id: EmoteSetId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub emotes: Vec<EmoteId>,
	pub capacity: Option<i32>,
	pub owner_id: Option<UserId>,
	pub origins: Vec<EmoteSetId>,
	pub kind: EmoteSetKind,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::emote_set::EmoteSet> for EmoteSet {
	fn from(value: database::emote_set::EmoteSet) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			emotes: value.emotes.into_iter().map(|emote| emote.id).collect(),
			capacity: value.capacity,
			owner_id: value.owner_id,
			origins: value
				.origin_config
				.into_iter()
				.flat_map(|oc| oc.origins)
				.map(|o| o.id)
				.collect(),
			kind: value.kind,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<EmoteSet>()]
}
