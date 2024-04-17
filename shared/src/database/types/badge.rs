use std::sync::Arc;

use bson::oid::ObjectId;

use super::{FileSet, FileSetKind, FileSetProperties};
use crate::database::Collection;
use crate::types::old::{CosmeticBadgeModel, ImageFile, ImageFormat, ImageHost, ImageHostKind};

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub file_set_id: ObjectId,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Collection for Badge {
	const COLLECTION_NAME: &'static str = "badges";
}

impl Badge {
	#[tracing::instrument(level = "info", skip(self), fields(badge_id = %self.id))]
	pub fn into_old_model(self, file_set: &FileSet, cdn_base_url: &str) -> Option<CosmeticBadgeModel> {
		if file_set.kind != FileSetKind::Badge {
			tracing::error!("Badge file set kind is not of type Badge");
			return None;
		}

		let host = ImageHost::new(
			cdn_base_url,
			ImageHostKind::Badge,
			self.id,
			file_set.properties.as_old_image_files(),
		);

		Some(CosmeticBadgeModel {
			id: self.id,
			name: self.name,
			tag: self.tags.into_iter().next().unwrap_or_default(),
			tooltip: self.description,
			host,
		})
	}
}
