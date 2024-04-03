use ulid::Ulid;

use super::cosmetic::{CosmeticBadgeModel, CosmeticPaintModel};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct UserPartialModel {
	pub id: Ulid,
	#[serde(rename = "type", skip_serializing_if = "String::is_empty")]
	pub ty: String,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub avatar_url: Option<String>,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub roles: Vec<Ulid>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionPartial>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct UserConnectionPartial {
	pub id: Ulid,
	pub platform: String,
	pub username: String,
	pub display_name: String,
	pub linked_at: u64,
	pub emote_capacity: i32,
	pub emote_set_id: Option<Ulid>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct UserStyle {
	pub color: i32,
	pub paint_id: Option<Ulid>,
	pub paint: Option<CosmeticPaintModel>,
	pub badge_id: Option<Ulid>,
	pub badge: Option<CosmeticBadgeModel>,
}
