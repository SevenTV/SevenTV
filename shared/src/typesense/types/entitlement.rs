use super::TypesenseGenericCollection;
use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString, EntitlementGroupId};
use crate::typesense::types::TypesenseCollection;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "entitlement_groups")]
#[serde(deny_unknown_fields)]
pub struct EntitlementGroup {
	pub id: EntitlementGroupId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub entitlement_grants: Vec<EntitlementEdgeKindString>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl EntitlementGroup {
	pub fn from_db(
		value: crate::database::entitlement::EntitlementGroup,
		entitlement_grants: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			entitlement_grants: entitlement_grants.into_iter().map(Into::into).collect(),
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: chrono::Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<EntitlementGroup>()]
}
