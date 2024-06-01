use std::sync::Arc;

use super::ImageSet;
use crate::database::{Collection, Id};

pub type BadgeId = Id<Badge>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
	#[serde(rename = "_id")]
	pub id: BadgeId,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub image_set: ImageSet,
}

impl Collection for Badge {
	const COLLECTION_NAME: &'static str = "badges";
}
