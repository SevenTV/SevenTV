use bitmask_enum::bitmask;

use super::{
	is_default, CosmeticBadgeModel, CosmeticPaintModel, EmoteSetPartialModel, UserConnectionModel,
	UserConnectionPartialModel,
};
use crate::database::{BadgeId, PaintId, RoleId, UserId};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L15
pub struct UserModel {
	pub id: UserId,
	#[serde(skip_serializing_if = "is_default")]
	pub user_type: UserTypeModel,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "is_default")]
	pub created_at: u64,
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
	pub roles: Vec<RoleId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L30
pub struct UserPartialModel {
	pub id: UserId,
	#[serde(skip_serializing_if = "is_default")]
	pub user_type: UserTypeModel,
	pub username: String,
	pub display_name: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub avatar_url: String,
	pub style: UserStyle,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub role_ids: Vec<RoleId>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<UserConnectionPartialModel>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L41
pub struct UserStyle {
	#[serde(skip_serializing_if = "is_default")]
	pub color: i32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub paint_id: Option<PaintId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub paint: Option<CosmeticPaintModel>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub badge_id: Option<BadgeId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub badge: Option<CosmeticBadgeModel>,
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L49
pub enum UserTypeModel {
	#[default]
	Regular,
	Bot,
	System,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/user.model.go#L171
pub struct UserEditorModel {
	pub id: UserId,
	pub permissions: UserEditorModelPermission,
	pub visible: bool,
	pub added_at: u64,
}

#[bitmask(u8)]
pub enum UserEditorModelPermission {
	ModifyEmotes = 1 << 0,
	UsePrivateEmotes = 1 << 1,
	ManageProfile = 1 << 2,
	ManageOwnedEmotes = 1 << 3,
	ManageEmoteSets = 1 << 4,
	ManageBilling = 1 << 5,
	ManageEditors = 1 << 6,
	ViewMessages = 1 << 7,
}

impl Default for UserEditorModelPermission {
	fn default() -> Self {
		UserEditorModelPermission::none()
	}
}

impl serde::Serialize for UserEditorModelPermission {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.bits().serialize(serializer)
	}
}

impl<'a> serde::Deserialize<'a> for UserEditorModelPermission {
	fn deserialize<D>(deserializer: D) -> Result<UserEditorModelPermission, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let bits = u8::deserialize(deserializer)?;
		Ok(UserEditorModelPermission::from(bits))
	}
}
