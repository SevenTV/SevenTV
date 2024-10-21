use chrono::Utc;
use macros::TypesenseCollection;
use serde::{Deserialize, Serialize};

use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString};
use crate::database::product::special_event::SpecialEventId;
use crate::database::user::UserId;
use crate::database::{self};
use crate::typesense::types::TypesenseGenericCollection;

#[derive(Debug, Clone, Serialize, Deserialize, TypesenseCollection)]
#[typesense(collection_name = "special_events")]
#[serde(deny_unknown_fields)]
pub struct SpecialEvent {
	pub id: SpecialEventId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	pub entitlements: Vec<EntitlementEdgeKindString>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl SpecialEvent {
	pub fn from_db(
		value: database::product::special_event::SpecialEvent,
		entitlements: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			created_by: value.created_by,
			entitlements: entitlements.into_iter().map(EntitlementEdgeKindString).collect(),
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<SpecialEvent>()]
}
