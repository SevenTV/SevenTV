use bson::oid::ObjectId;

use super::UserPartialModel;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/entitlement.model.go#L9
pub struct EntitlementModel {
	pub id: ObjectId,
	pub kind: EntitlementKind,
	pub user: UserPartialModel,
	pub ref_id: ObjectId,
}

#[derive(Debug, Clone, Default, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// https://github.com/SevenTV/API/blob/6d36bb52c8f7731979882db553e8dbc0153a38bf/data/model/entitlement.model.go#L16
pub enum EntitlementKind {
	#[default]
	Badge,
	Paint,
	EmoteSet,
}
