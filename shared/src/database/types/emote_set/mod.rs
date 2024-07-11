use crate::database::{Collection, Id};

mod emote;
mod origin;

use derive_builder::Builder;
pub use emote::*;
pub use origin::*;

use super::user::UserId;
use super::GenericCollection;

pub type EmoteSetId = Id<EmoteSet>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct EmoteSet {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: EmoteSetId,
	pub name: String,
	#[builder(default)]
	pub description: Option<String>,
	#[builder(default)]
	pub tags: Vec<String>,
	#[builder(default)]
	pub emotes: Vec<EmoteSetEmote>,
	pub capacity: Option<i32>,
	pub owner_id: Option<UserId>,
	#[builder(default)]
	pub origin_config: Option<EmoteSetOriginConfig>,
	#[builder(default)]
	pub kind: EmoteSetKind,
}

impl Collection for EmoteSet {
	const COLLECTION_NAME: &'static str = "emote_sets";

	fn indexes() -> Vec<mongodb::IndexModel> {
		vec![
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"owner_id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"emotes.id": 1,
				})
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id": 1,
					"emotes.id": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
			mongodb::IndexModel::builder()
				.keys(mongodb::bson::doc! {
					"_id": 1,
					"emotes.alias": 1,
				})
				.options(mongodb::options::IndexOptions::builder().unique(true).build())
				.build(),
		]
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[serde(deny_unknown_fields)]
#[repr(u8)]
pub enum EmoteSetKind {
	#[default]
	Normal = 0,
	Personal = 1,
	Global = 2,
	Special = 3,
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<EmoteSet>()]
}
