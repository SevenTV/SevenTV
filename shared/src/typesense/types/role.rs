use super::TypesenseGenericCollection;
use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString};
use crate::database::role::RoleId;
use crate::database::user::UserId;
use crate::typesense::types::TypesenseCollection;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "roles")]
#[serde(deny_unknown_fields)]
pub struct Role {
	pub id: RoleId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub hoist: bool,
	pub color: Option<i32>,
	pub created_by: UserId,
	pub entitlement_grants: Vec<EntitlementEdgeKindString>,
	pub rank: i32,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl Role {
	pub fn from_db(
		value: crate::database::role::Role,
		entitlement_grants: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			hoist: value.hoist,
			created_by: value.created_by,
			entitlement_grants: entitlement_grants.into_iter().map(Into::into).collect(),
			color: value.color,
			rank: value.rank,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: chrono::Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Role>()]
}
