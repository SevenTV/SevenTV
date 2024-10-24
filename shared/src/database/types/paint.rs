use super::image_set::ImageSet;
use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

pub type PaintId = Id<Paint>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection, PartialEq)]
#[mongo(collection_name = "paints")]
#[mongo(index(fields(search_updated_at = 1)))]
#[mongo(index(fields(created_by = 1)))]
#[mongo(search = "crate::typesense::types::paint::Paint")]
#[serde(deny_unknown_fields)]
pub struct Paint {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: PaintId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub data: PaintData,
	pub created_by: UserId,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
	#[serde(with = "crate::database::serde")]
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PaintData {
	pub layers: Vec<PaintLayer>,
	pub shadows: Vec<PaintShadow>,
}

pub type PaintLayerId = Id<PaintLayer>;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PaintLayer {
	pub id: PaintLayerId,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum PaintLayerType {
	SingleColor(i32),
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PaintGradientStop {
	pub at: f64,
	pub color: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum PaintRadialGradientShape {
	#[default]
	Ellipse,
	Circle,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PaintShadow {
	pub color: i32,
	pub offset_x: f64,
	pub offset_y: f64,
	pub blur: f64,
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[MongoGenericCollection::new::<Paint>()]
}
