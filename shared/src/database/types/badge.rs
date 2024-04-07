use std::sync::Arc;

use crate::types::old::{CosmeticBadgeModel, ImageFile, ImageFormat, ImageHost, ImageHostKind};

use super::{FileSet, FileSetKind, FileSetProperties};
use crate::database::Table;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Badge {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub file_set_id: ulid::Ulid,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for Badge {
	const TABLE_NAME: &'static str = "badges";
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
