use std::sync::Arc;

use shared::{object_id::ObjectId, types::{CosmeticPaint, ImageHost}};

use crate::{database::Table, global::Global};

use super::ImageFileData;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Paint {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: PaintData,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Table for Paint {
	const TABLE_NAME: &'static str = "paints";
}

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct PaintFile {
	pub paint_id: ulid::Ulid,
	pub file_id: ulid::Ulid,
	#[from_row(from_fn = "scuffle_utils::database::json")]
	pub data: ImageFileData,
}

impl Table for PaintFile {
	const TABLE_NAME: &'static str = "paint_files";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(default)]
pub struct PaintData {}

impl Paint {
	pub async fn into_cosmetic_paint(self, global: &Arc<Global>) -> Result<CosmeticPaint, ()> {
		let badge_files: Vec<PaintFile> = scuffle_utils::database::query("SELECT * FROM paint_files WHERE paint_id = $1")
			.bind(self.id)
			.build_query_as()
			.fetch_all(&global.db())
			.await
			.map_err(|_| ())?;
		let files = global.file_by_id_loader().load_many(badge_files.iter().map(|f| f.file_id)).await?;

		let host = ImageHost {
			url: format!("{}/badge/{}", global.config().api.cdn_base_url, ObjectId::from_ulid(self.id)),
			files: badge_files
				.into_iter()
				.filter_map(|f| {
					let file = files.get(&f.file_id)?;
					Some(f.data.into_host_file(file.path.clone()))
				})
				.collect()
		};

		Ok(CosmeticPaint {
			id: self.id.into(),
			name: self.name,
			tag: self.tags.into_iter().next().unwrap_or_default(),
			tooltip: self.description,
			host,
		})
	}
}
