use super::{is_default, ImageHost, ImageHostKind, UserPartialModel};
use crate::database::{BadgeId, Paint, PaintId, PaintLayerType, PaintRadialGradientShape, PaintShadow, UserId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[aliases(
	CosmeticModelPaint = CosmeticModel<CosmeticPaintModel>,
	CosmeticModelBadge = CosmeticModel<CosmeticBadgeModel>,
	CosmeticModelAvatar = CosmeticModel<CosmeticAvatarModel>,
)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/cosmetic.model.go#L15
pub struct CosmeticModel<T: CosmeticModelData> {
	pub id: T::Id,
	pub kind: CosmeticKind,
	pub data: T,
}

pub trait CosmeticModelData {
	type Id;
}

impl CosmeticModelData for CosmeticPaintModel {
	type Id = PaintId;
}

impl CosmeticModelData for CosmeticBadgeModel {
	type Id = BadgeId;
}

impl CosmeticModelData for CosmeticAvatarModel {
	type Id = UserId;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/cosmetic.model.go#L21
pub enum CosmeticKind {
	Badge,
	Paint,
	Avatar,
}

async_graphql::scalar!(CosmeticKind);

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[graphql(name = "CosmeticPaint", rename_fields = "snake_case")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/cosmetic.model.go#L29
pub struct CosmeticPaintModel {
	pub id: PaintId,
	pub name: String,
	pub color: Option<i32>,
	pub gradients: Vec<CosmeticPaintGradient>,
	pub shadows: Vec<CosmeticPaintShadow>,
	pub text: Option<CosmeticPaintText>,
	#[graphql(deprecation)]
	pub function: CosmeticPaintFunction,
	#[graphql(deprecation)]
	pub repeat: bool,
	#[graphql(deprecation)]
	pub angle: i32,
	#[graphql(deprecation)]
	pub shape: CosmeticPaintShape,
	#[graphql(deprecation)]
	pub image_url: String,
	#[graphql(deprecation)]
	pub stops: Vec<CosmeticPaintGradientStop>,
}

impl CosmeticPaintModel {
	pub fn from_db(value: Paint, cdn_base_url: &str) -> Option<Self> {
		let first_layer = value.data.layers.first();

		Some(Self {
			id: value.id,
			name: value.name,
			color: first_layer.and_then(|l| match l.ty {
				PaintLayerType::SingleColor(c) => Some(c as i32),
				_ => None,
			}),
			gradients: vec![],
			shadows: value.data.shadows.into_iter().map(|s| s.into()).collect(),
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
				.and_then(|l| match &l.ty {
					PaintLayerType::Image(image_set) => image_set.outputs.first().map(|i| i.get_url(cdn_base_url)),
					_ => None,
				})
				.unwrap_or_default(),
			stops: first_layer
				.and_then(|l| match &l.ty {
					PaintLayerType::LinearGradient { stops, .. } | PaintLayerType::RadialGradient { stops, .. } => Some(
						stops
							.iter()
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[graphql(rename_fields = "snake_case")]
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

async_graphql::scalar!(CosmeticPaintFunction);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[graphql(name = "CosmeticPaintStop", rename_fields = "snake_case")]
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

async_graphql::scalar!(CosmeticPaintCanvasRepeat);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintShadow {
	pub x_offset: f64,
	pub y_offset: f64,
	pub radius: f64,
	pub color: i32,
}

impl From<PaintShadow> for CosmeticPaintShadow {
	fn from(s: PaintShadow) -> Self {
		Self {
			color: s.color as i32,
			x_offset: s.offset_x,
			y_offset: s.offset_y,
			radius: s.blur,
		}
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[graphql(rename_fields = "snake_case")]
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[graphql(rename_fields = "snake_case")]
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

async_graphql::scalar!(CosmeticPaintTextTransform);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CosmeticPaintShape {
	#[default]
	Circle,
	Ellipse,
}

async_graphql::scalar!(CosmeticPaintShape);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema, async_graphql::SimpleObject)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[graphql(name = "CosmeticBadge", rename_fields = "snake_case")]
pub struct CosmeticBadgeModel {
	pub id: BadgeId,
	pub name: String,
	pub tag: String,
	pub tooltip: String,
	pub host: ImageHost,
}

impl CosmeticBadgeModel {
	pub fn from_db(value: crate::database::Badge, cdn_base_url: &str) -> Option<Self> {
		let id = value.id.cast();
		let host = ImageHost::from_image_set(&value.image_set, cdn_base_url, ImageHostKind::Badge, &id);

		Some(Self {
			id,
			name: value.name,
			tag: value.tags.into_iter().next().unwrap_or_default(),
			tooltip: value.description,
			host,
		})
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CosmeticAvatarModel {
	pub id: UserId,
	pub user: UserPartialModel,
	#[serde(skip_serializing_if = "is_default", rename = "as")]
	pub aas: String,
	pub host: ImageHost,
}