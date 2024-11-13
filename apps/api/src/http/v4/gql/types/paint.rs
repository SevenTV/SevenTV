use shared::database::paint::{PaintId, PaintLayerId};
use shared::database::user::UserId;

use super::{Color, Image};

#[derive(async_graphql::SimpleObject)]
pub struct Paint {
	pub id: PaintId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub data: PaintData,
	pub created_by_id: UserId,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Paint {
	pub fn from_db(value: shared::database::paint::Paint, cdn_base_url: &url::Url) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			data: PaintData::from_db(value.data, cdn_base_url),
			created_by_id: value.created_by,
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintData {
	pub layers: Vec<PaintLayer>,
	pub shadows: Vec<PaintShadow>,
}

impl PaintData {
	pub fn from_db(value: shared::database::paint::PaintData, cdn_base_url: &url::Url) -> Self {
		Self {
			layers: value
				.layers
				.into_iter()
				.map(|l| PaintLayer::from_db(l, cdn_base_url))
				.collect(),
			shadows: value.shadows.into_iter().map(Into::into).collect(),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintLayer {
	pub id: PaintLayerId,
	pub ty: PaintLayerType,
	pub opacity: f64,
}

impl PaintLayer {
	pub fn from_db(value: shared::database::paint::PaintLayer, cdn_base_url: &url::Url) -> Self {
		Self {
			id: value.id,
			ty: PaintLayerType::from_db(value.ty, cdn_base_url),
			opacity: value.opacity,
		}
	}
}

#[derive(async_graphql::Union)]
pub enum PaintLayerType {
	SingleColor(PaintLayerTypeSingleColor),
	LinearGradient(PaintLayerTypeLinearGradient),
	RadialGradient(PaintLayerTypeRadialGradient),
	Image(PaintLayerTypeImage),
}

impl PaintLayerType {
	pub fn from_db(value: shared::database::paint::PaintLayerType, cdn_base_url: &url::Url) -> Self {
		match value {
			shared::database::paint::PaintLayerType::SingleColor(color) => {
				Self::SingleColor(PaintLayerTypeSingleColor { color: color.into() })
			}
			shared::database::paint::PaintLayerType::LinearGradient { angle, repeating, stops } => {
				Self::LinearGradient(PaintLayerTypeLinearGradient {
					angle,
					repeating,
					stops: stops.into_iter().map(Into::into).collect(),
				})
			}
			shared::database::paint::PaintLayerType::RadialGradient { repeating, stops, shape } => {
				Self::RadialGradient(PaintLayerTypeRadialGradient {
					repeating,
					stops: stops.into_iter().map(Into::into).collect(),
					shape: shape.into(),
				})
			}
			shared::database::paint::PaintLayerType::Image(images) => Self::Image(PaintLayerTypeImage {
				images: images.outputs.into_iter().map(|o| Image::from_db(o, cdn_base_url)).collect(),
			}),
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintLayerTypeSingleColor {
	pub color: Color,
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintLayerTypeLinearGradient {
	pub angle: i32,
	pub repeating: bool,
	pub stops: Vec<PaintGradientStop>,
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintLayerTypeRadialGradient {
	pub repeating: bool,
	pub stops: Vec<PaintGradientStop>,
	pub shape: PaintRadialGradientShape,
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintGradientStop {
	pub at: f64,
	pub color: Color,
}

impl From<shared::database::paint::PaintGradientStop> for PaintGradientStop {
	fn from(value: shared::database::paint::PaintGradientStop) -> Self {
		Self {
			at: value.at,
			color: value.color.into(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum PaintRadialGradientShape {
	Ellipse,
	Circle,
}

impl From<shared::database::paint::PaintRadialGradientShape> for PaintRadialGradientShape {
	fn from(value: shared::database::paint::PaintRadialGradientShape) -> Self {
		match value {
			shared::database::paint::PaintRadialGradientShape::Ellipse => Self::Ellipse,
			shared::database::paint::PaintRadialGradientShape::Circle => Self::Circle,
		}
	}
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintLayerTypeImage {
	pub images: Vec<Image>,
}

#[derive(async_graphql::SimpleObject)]
pub struct PaintShadow {
	pub color: Color,
	pub offset_x: f64,
	pub offset_y: f64,
	pub blur: f64,
}

impl From<shared::database::paint::PaintShadow> for PaintShadow {
	fn from(value: shared::database::paint::PaintShadow) -> Self {
		Self {
			color: value.color.into(),
			offset_x: value.offset_x,
			offset_y: value.offset_y,
			blur: value.blur,
		}
	}
}
