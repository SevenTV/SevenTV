use std::collections::HashMap;
use std::sync::Arc;

use crate::database::{Collection, Id};

use super::ImageSet;

pub type PaintId = Id<Paint>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Paint {
	#[serde(rename = "_id")]
	pub id: PaintId,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub data: PaintData,
}

impl Collection for Paint {
	const COLLECTION_NAME: &'static str = "paints";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PaintData {
	pub layers: Vec<PaintLayer>,
	pub shadows: Vec<PaintShadow>,
}

pub type PaintLayerId = Id<PaintLayer>;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PaintLayer {
	pub id: PaintLayerId,
	#[serde(flatten)]
	pub ty: PaintLayerType,
	pub opacity: f64,
}

impl Default for PaintLayer {
	fn default() -> Self {
		Self {
			id: PaintLayerId::default(),
			ty: PaintLayerType::default(),
			opacity: 1.0,
		}
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
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
	Image(ImageSet),
}

impl Default for PaintLayerType {
	fn default() -> Self {
		Self::SingleColor(0xffffff)
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PaintGradientStop {
	pub at: f64,
	pub color: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum PaintRadialGradientShape {
	#[default]
	Ellipse,
	Circle,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PaintShadow {
	pub color: u32,
	pub offset_x: f64,
	pub offset_y: f64,
	pub blur: f64,
}
