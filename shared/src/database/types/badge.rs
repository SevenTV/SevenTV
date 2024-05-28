use std::sync::Arc;

use crate::database::{Collection, Id};
use crate::types::old::{CosmeticBadgeModel, ImageFile, ImageFormat, ImageHost, ImageHostKind};

use super::ImageSet;

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

impl Badge {
	#[tracing::instrument(level = "info", skip(self), fields(badge_id = %self.id))]
	pub fn into_old_model(self, cdn_base_url: &str) -> Option<CosmeticBadgeModel> {
		let id = self.id.cast();
		let host = ImageHost::from_image_set(&self.image_set, cdn_base_url, ImageHostKind::Badge, &id);

		Some(CosmeticBadgeModel {
			id,
			name: self.name,
			tag: self.tags.into_iter().next().unwrap_or_default(),
			tooltip: self.description,
			host,
		})
	}
}
