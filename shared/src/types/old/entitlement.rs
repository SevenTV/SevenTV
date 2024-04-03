use ulid::Ulid;

use super::UserPartialModel;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EntitlementModel {
    pub id: Ulid,
    pub kind: EntitlementKind,
    pub user: UserPartialModel,
    pub ref_id: Ulid,
}

#[derive(Debug, Clone, Default, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntitlementKind {
    #[default]
    Badge,
    Paint,
    EmoteSet,
}
