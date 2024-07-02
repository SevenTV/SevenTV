use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString};
use crate::database::product::promotion::{PromotionId, PromotionTrigger};
use crate::database::product::ProductId;
use crate::database::user::UserId;
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Serialize, Deserialize, Clone, TypesenseCollection)]
#[typesense(collection_name = "promotions")]
#[serde(deny_unknown_fields)]
pub struct Promotion {
	pub id: PromotionId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub products: Vec<ProductId>,
	pub active_from: i64,
	pub active_to: i64,
	pub unit_threshold: i32,
	pub public: bool,
	pub created_by: UserId,
	pub trigger: PromotionTrigger,
	pub entitlement_grants: Vec<EntitlementEdgeKindString>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl Promotion {
	pub fn from_db(
		value: database::product::promotion::Promotion,
		entitlement_grants: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			products: value.products.into_iter().map(|product| product.id).collect(),
			active_from: value.time_period.start.timestamp_millis(),
			active_to: value.time_period.end.timestamp_millis(),
			unit_threshold: value.unit_threshold,
			public: value.public,
			created_by: value.created_by,
			trigger: value.trigger,
			entitlement_grants: entitlement_grants.into_iter().map(Into::into).collect(),
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<Promotion>()]
}
