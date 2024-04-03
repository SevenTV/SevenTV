use ulid::Ulid;

use super::is_default;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RoleModel {
    pub id: Ulid,
    pub name: String,
    pub position: i32,
    pub color: i32,
    pub allowed: String,
    pub denied: String,
    #[serde(skip_serializing_if = "is_default")]
    pub invisible: bool,
}
