use mongodb::bson::oid::ObjectId;
use shared::database;

#[derive(Debug, serde::Deserialize)]
pub struct Cosmetic {
	#[serde(rename = "_id")]
	pub id: ObjectId,
	pub name: String,
	#[serde(flatten)]
	pub data: CosmeticData,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CosmeticData {
	Badge {
		tooltip: String,
		tag: Option<String>,
	},
	Paint {
		#[serde(flatten)]
		data: PaintData,
		#[serde(default)]
		drop_shadows: Vec<PaintDropShadow>,
	},
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "function", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaintData {
	LinearGradient {
		stops: Vec<GradientStop>,
		repeat: bool,
		angle: i32,
	},
	RadialGradient {
		stops: Vec<GradientStop>,
		repeat: bool,
		angle: i32,
		shape: database::paint::PaintRadialGradientShape,
	},
	Url {
		image_url: Option<String>,
	},
}

#[derive(Debug, serde::Deserialize)]
pub struct GradientStop {
	pub at: f64,
	pub color: i32,
}

impl From<GradientStop> for database::paint::PaintGradientStop {
	fn from(value: GradientStop) -> Self {
		Self {
			at: value.at,
			color: value.color,
		}
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct PaintDropShadow {
	pub x_offset: f64,
	pub y_offset: f64,
	pub radius: f64,
	pub color: i32,
}

impl From<PaintDropShadow> for database::paint::PaintShadow {
	fn from(value: PaintDropShadow) -> Self {
		Self {
			color: value.color,
			offset_x: value.x_offset,
			offset_y: value.y_offset,
			blur: value.radius,
		}
	}
}
