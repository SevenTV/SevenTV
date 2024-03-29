use std::sync::Arc;

use shared::object_id::ObjectId;
use shared::types::old::{
	CosmeticPaint, CosmeticPaintFunction, CosmeticPaintGradientStop, CosmeticPaintShadow, CosmeticPaintShape, ImageHost,
};

use super::ImageFileData;
use crate::database::Table;
use crate::global::Global;

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
pub struct PaintData {
	layers: Vec<PaintLayer>,
	shadows: Vec<PaintShadow>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(default)]
pub struct PaintLayer {
	#[serde(flatten)]
	pub ty: PaintLayerType,
	pub opacity: f64,
}

impl Default for PaintLayer {
	fn default() -> Self {
		Self {
			ty: PaintLayerType::default(),
			opacity: 1.0,
		}
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PaintLayerType {
	SingleColor(u32),
	LinearGradient {
		angle: i32,
		repeating: bool,
		stops: Vec<PaintGradientStop>,
	},
	RadialGradient {
		angle: i32,
		repeating: bool,
		stops: Vec<PaintGradientStop>,
		shape: PaintRadialGradientShape,
	},
	Image(ulid::Ulid),
}

impl Default for PaintLayerType {
	fn default() -> Self {
		Self::SingleColor(0xffffff)
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct PaintGradientStop {
	pub at: f64,
	pub color: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaintRadialGradientShape {
	#[default]
	Ellipse,
	Circle,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct PaintShadow {
	pub color: u32,
	pub offset_x: i32,
	pub offset_y: i32,
	pub blur: i32,
}

impl From<PaintShadow> for CosmeticPaintShadow {
	fn from(s: PaintShadow) -> Self {
		Self {
			color: s.color as i32,
			x_offset: s.offset_x as f64,
			y_offset: s.offset_y as f64,
			radius: s.blur as f64,
		}
	}
}

impl Paint {
	pub async fn into_old_model(self, global: &Arc<Global>) -> Result<CosmeticPaint, ()> {
		let paint_files: Vec<PaintFile> = scuffle_utils::database::query("SELECT * FROM paint_files WHERE paint_id = $1")
			.bind(self.id)
			.build_query_as()
			.fetch_all(&global.db())
			.await
			.map_err(|_| ())?;
		let files = global
			.file_by_id_loader()
			.load_many(paint_files.iter().map(|f| f.file_id))
			.await?;

		let first_layer = self.data.layers.first();

		Ok(CosmeticPaint {
			id: self.id.into(),
			name: self.name,
			color: first_layer.and_then(|l| match l.ty {
				PaintLayerType::SingleColor(c) => Some(c as i32),
				_ => None,
			}),
			gradients: vec![],
			shadows: self.data.shadows.into_iter().map(|s| s.into()).collect(),
			text: None,
			function: first_layer
				.map(|l| match l.ty {
					PaintLayerType::SingleColor(..) => CosmeticPaintFunction::LinearGradient,
					PaintLayerType::LinearGradient { .. } => CosmeticPaintFunction::LinearGradient,
					PaintLayerType::RadialGradient { .. } => CosmeticPaintFunction::RadialGradient,
					PaintLayerType::Image(..) => CosmeticPaintFunction::Url,
				})
				.unwrap_or(CosmeticPaintFunction::LinearGradient),
			repeat: first_layer
				.map(|l| match l.ty {
					PaintLayerType::LinearGradient { repeating, .. } | PaintLayerType::RadialGradient { repeating, .. } => {
						repeating
					}
					_ => false,
				})
				.unwrap_or_default(),
			angle: first_layer
				.and_then(|l| match l.ty {
					PaintLayerType::LinearGradient { angle, .. } | PaintLayerType::RadialGradient { angle, .. } => {
						Some(angle)
					}
					_ => None,
				})
				.unwrap_or_default(),
			shape: first_layer
				.and_then(|l| match l.ty {
					PaintLayerType::RadialGradient {
						shape: PaintRadialGradientShape::Ellipse,
						..
					} => Some(CosmeticPaintShape::Ellipse),
					PaintLayerType::RadialGradient {
						shape: PaintRadialGradientShape::Circle,
						..
					} => Some(CosmeticPaintShape::Circle),
					_ => None,
				})
				.unwrap_or_default(),
			image_url: first_layer
				.and_then(|l| match l.ty {
					PaintLayerType::Image(id) => files.get(&id).map(|f| f.path.clone()),
					_ => None,
				})
				.unwrap_or_default(),
			stops: first_layer
				.and_then(|l| match &l.ty {
					PaintLayerType::LinearGradient { stops, .. } | PaintLayerType::RadialGradient { stops, .. } => Some(
						stops
							.into_iter()
							.map(|s| CosmeticPaintGradientStop {
								color: s.color as i32,
								at: s.at,
								center_at: [0.0, 0.0],
							})
							.collect(),
					),
					_ => None,
				})
				.unwrap_or_default(),
		})
	}
}
