use super::is_default;
use crate::database::RoleId;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/role.model.go#L10
pub struct RoleModel {
	pub id: RoleId,
	pub name: String,
	pub position: i32,
	pub color: i32,
	pub allowed: String,
	pub denied: String,
	#[serde(skip_serializing_if = "is_default")]
	pub invisible: bool,
}
