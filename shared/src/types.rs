use crate::object_id::ObjectId;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct UserModelPartial {
	pub id: ObjectId,
	#[serde(rename = "type")]
	pub ty: String,
	pub username: String,
	pub display_name: String,
	pub avatar_url: String,
	pub style: UserStyle,
	pub roles: Vec<ObjectId>,
	pub connections: Vec<UserConnectionPartial>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct UserStyle {
	pub color: i32,
	pub paint_id: Option<ObjectId>,
	pub badge_id: Option<ObjectId>,
	pub paint: Option<CosmeticPaint>,
	pub badge: Option<CosmeticBadgeModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CosmeticBadgeModel {
	pub id: ObjectId,
	pub name: String,
	pub tag: String,
	pub tooltip: String,
	pub host: ImageHost,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CosmeticPaint {
	pub id: ObjectId,
	pub name: String,
	pub color: Option<i32>,
	pub gradients: Vec<CosmeticPaintGradient>,
	pub shadows: Vec<CosmeticPaintShadow>,
	pub text: Option<CosmeticPaintText>,
	pub function: CosmeticPaintFunction,
	pub repeat: bool,
	pub angle: i32,
	pub shape: String,
	pub image_url: String,
	pub stops: Vec<CosmeticPaintGradientStop>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CosmeticPaintGradient {
	pub function: CosmeticPaintFunction,
	pub canvas_repeat: CosmeticPaintCanvasRepeat,
	pub canvas_size: [i32; 2],
	pub at: [i32; 2],
	pub stops: Vec<CosmeticPaintGradientStop>,
	pub image_url: String,
	pub shape: String,
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
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CosmeticPaintGradientStop {
	pub at: f64,
	pub color: i32,
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
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CosmeticPaintShadow {
	pub x_offset: f64,
	pub y_offset: f64,
	pub radius: f64,
	pub color: i32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CosmeticPaintText {
	pub weight: u8,
	pub shadows: Vec<CosmeticPaintShadow>,
	pub transform: Option<CosmeticPaintTextTransform>,
	pub stroke: Option<CosmeticPaintStroke>,
	pub variant: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CosmeticPaintTextTransform {
	Uppercase,
	Lowercase,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CosmeticPaintStroke {
	pub color: i32,
	pub width: f64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct UserConnectionPartial {
	pub id: String,
	pub platform: String,
	pub username: String,
	pub display_name: String,
	pub linked_at: i64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<ObjectId>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct ImageHost {
	pub url: String,
	pub files: Vec<ImageHostFile>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct ImageHostFile {
	pub name: String,
	pub static_name: String,
	pub width: i32,
	pub height: i32,
	pub frame_count: i32,
	pub size: i64,
	pub format: String,
}
