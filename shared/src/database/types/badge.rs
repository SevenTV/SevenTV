use derive_builder::Builder;

use super::image_set::ImageSet;
use super::GenericCollection;
use crate::database::{Collection, Id};

pub type BadgeId = Id<Badge>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct Badge {
	#[serde(rename = "_id")]
	#[builder(default)]
	pub id: BadgeId,
	pub name: String,
	#[builder(default)]
	pub description: Option<String>,
	#[builder(default)]
	pub tags: Vec<String>,
	pub image_set: ImageSet,
}

impl Collection for Badge {
	const COLLECTION_NAME: &'static str = "badges";
}

pub(super) fn collections() -> impl IntoIterator<Item = GenericCollection> {
	[GenericCollection::new::<Badge>()]
}
