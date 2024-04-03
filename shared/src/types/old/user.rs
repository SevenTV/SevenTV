use ulid::Ulid;

use super::{is_default, CosmeticBadgeModel, CosmeticPaintModel, EmoteSetPartialModel, UserConnectionModel, UserConnectionPartialModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct UserModel {
	pub id: Ulid,
	#[serde(skip_serializing_if = "is_default")]
	pub user_type: UserTypeModel,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "is_default")]
	pub created_at: i64,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub avatar_url: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub biography: String,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub emote_sets: Vec<EmoteSetPartialModel>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub editors: Vec<UserEditorModel>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub role_ids: Vec<Ulid>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct UserPartialModel {
	pub id: Ulid,
	#[serde(skip_serializing_if = "is_default")]
	pub user_type: UserTypeModel,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub avatar_url: String,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub role_ids: Vec<Ulid>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionPartialModel>,
}

#[derive(Debug, Clone,  Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct UserStyle {
	#[serde(skip_serializing_if = "is_default")]
	pub color: i32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub paint_id: Option<Ulid>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub paint: Option<CosmeticPaintModel>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub badge_id: Option<Ulid>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub badge: Option<CosmeticBadgeModel>,
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq,  serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserTypeModel {
	#[default]
	#[serde(rename = "")]
	Regular,
	Bot,
	System,
}

#[derive(Debug, Clone,  Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct UserEditorModel {
	pub id: Ulid,
	pub permissions: i32,
	pub visible: bool,
	pub added_at: i64,
}
