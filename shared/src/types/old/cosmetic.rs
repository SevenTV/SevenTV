use ulid::Ulid;

use super::{is_default, ImageHost, UserPartialModel};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/cosmetic.model.go#L15
pub struct CosmeticModel<T: utoipa::ToSchema<'static>> {
	pub id: Ulid,
	pub kind: CosmeticKind,
	pub data: T,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/cosmetic.model.go#L21
pub enum CosmeticKind {
	Badge,
	Paint,
	Avatar,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/cosmetic.model.go#L29
pub struct CosmeticPaintModel {
	pub id: Ulid,
	pub name: String,
	pub color: Option<i32>,
	pub gradients: Vec<CosmeticPaintGradient>,
	pub shadows: Vec<CosmeticPaintShadow>,
	pub text: Option<CosmeticPaintText>,
	pub function: CosmeticPaintFunction,
	pub repeat: bool,
	pub angle: i32,
	pub shape: CosmeticPaintShape,
	pub image_url: String,
	pub stops: Vec<CosmeticPaintGradientStop>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticPaintGradient {
	pub function: CosmeticPaintFunction,
	pub canvas_repeat: CosmeticPaintCanvasRepeat,
	pub canvas_size: [f64; 2],
	#[serde(skip_serializing_if = "is_default")]
	pub at: [f64; 2],
	pub stops: Vec<CosmeticPaintGradientStop>,
	#[serde(skip_serializing_if = "is_default")]
	pub image_url: String,
	pub shape: Option<CosmeticPaintShape>,
	#[serde(skip_serializing_if = "is_default")]
	pub angle: i32,
	pub repeat: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CosmeticPaintFunction {
	#[default]
	LinearGradient,
	RadialGradient,
	Url,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticPaintGradientStop {
	pub at: f64,
	pub color: i32,
	#[serde(skip_serializing_if = "is_default")]
	pub center_at: [f64; 2],
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CosmeticPaintCanvasRepeat {
	#[default]
	NoRepeat,
	RepeatX,
	RepeatY,
	Revert,
	Round,
	Space,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticPaintShadow {
	pub x_offset: f64,
	pub y_offset: f64,
	pub radius: f64,
	pub color: i32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticPaintText {
	#[serde(skip_serializing_if = "is_default")]
	pub weight: u8,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub shadows: Vec<CosmeticPaintShadow>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub transform: Option<CosmeticPaintTextTransform>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stroke: Option<CosmeticPaintStroke>,
	pub variant: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticPaintStroke {
	pub color: i32,
	pub width: f64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CosmeticPaintTextTransform {
	#[default]
	Uppercase,
	Lowercase,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CosmeticPaintShape {
	#[default]
	Circle,
	Ellipse,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticBadgeModel {
	pub id: Ulid,
	pub name: String,
	pub tag: String,
	pub tooltip: String,
	pub host: ImageHost,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticAvatarModel {
	pub id: Ulid,
	pub user: UserPartialModel,
	#[serde(skip_serializing_if = "is_default", rename = "as")]
	pub aas: String,
	pub host: ImageHost,
}
