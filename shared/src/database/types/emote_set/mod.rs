use crate::database::{Id, MongoCollection};
use crate::typesense::types::impl_typesense_type;

mod emote;
mod origin;

pub use emote::*;
pub use origin::*;

use super::user::UserId;
use super::MongoGenericCollection;

pub type EmoteSetId = Id<EmoteSet>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq, Eq)]
#[mongo(collection_name = "emote_sets")]
#[mongo(index(fields("emotes.id" = 1)))]
#[mongo(index(fields(owner_id = 1)))]
#[mongo(index(fields(_id = 1, "emotes.id" = 1), unique))]
#[mongo(index(fields(_id = 1, "emotes.alias" = 1), unique))]
#[mongo(index(fields("origin_config.origins.id" = 1)))]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(search = "crate::typesense::types::emote_set::EmoteSet")]
#[serde(deny_unknown_fields)]
pub struct EmoteSet {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: EmoteSetId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub emotes: Vec<EmoteSetEmote>,
	pub capacity: Option<i32>,
	pub owner_id: Option<UserId>,
	pub origin_config: Option<EmoteSetOriginConfig>,
	pub kind: EmoteSetKind,
	pub emotes_changed_since_reindex: bool,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[serde(deny_unknown_fields)]
#[repr(u8)]
pub enum EmoteSetKind {
	/// Normal emote set
	Normal = 0,
	/// Personal emote set
	Personal = 1,
	/// Like a normal emote set but for multiple people
	Global = 2,
	/// Like a personal emote set but for multiple people
	Special = 3,
}

impl_typesense_type!(EmoteSetKind, Int32);

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<EmoteSet>()]
}
