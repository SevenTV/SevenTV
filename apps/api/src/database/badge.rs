use std::sync::Arc;

use shared::types::old::{CosmeticBadgeModel, ImageHost, ImageHostFile, ImageHostFormat, ImageHostKind};

use super::{FileSet, FileSetKind, FileSetProperties};
use crate::database::Table;
use crate::global::Global;

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
	#[tracing::instrument(level = "info", skip(self, global), fields(badge_id = %self.id))]
	pub async fn into_old_model(self, global: &Arc<Global>) -> Result<CosmeticBadgeModel, ()> {
		let file_set = global.file_set_by_id_loader().load(self.file_set_id).await?.ok_or(())?;

		if file_set.kind != FileSetKind::Badge {
			tracing::error!("Badge file set kind is not of type Badge");
			return Err(());
		}

		let host = ImageHost::new(
			&global.config().api.cdn_base_url,
			ImageHostKind::Badge,
			self.id,
			file_set.properties.as_old_image_files(),
		);

		Ok(CosmeticBadgeModel {
			id: self.id,
			name: self.name,
			tag: self.tags.into_iter().next().unwrap_or_default(),
			tooltip: self.description,
			host,
		})
	}
}
