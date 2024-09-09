use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::database::entitlement::EntitlementEdgeKindString;
use crate::database::product::codes::{CodeEffect, RedeemCodeId};
use crate::database::product::special_event::SpecialEventId;
use crate::database::user::UserId;
use crate::typesense::types::{TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, Serialize, Deserialize, TypesenseCollection)]
#[typesense(collection_name = "redeem_codes")]
#[serde(deny_unknown_fields)]
pub struct RedeemCode {
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub remaining_uses: i32,
	pub active_from: i64,
	pub active_to: i64,
	pub entitlements: Vec<EntitlementEdgeKindString>,
	pub created_by: UserId,
	pub special_event_id: Option<SpecialEventId>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::product::codes::RedeemCode> for RedeemCode {
	fn from(value: database::product::codes::RedeemCode) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			code: value.code,
			remaining_uses: value.remaining_uses,
			active_from: value.active_period.start.timestamp_millis(),
			active_to: value.active_period.end.timestamp_millis(),
			entitlements: match &value.effect {
				CodeEffect::DirectEntitlement { entitlements } => {
					entitlements.iter().cloned().map(EntitlementEdgeKindString).collect()
				}
				_ => vec![],
			},
			special_event_id: match &value.effect {
				CodeEffect::SpecialEvent { special_event_id } => Some(*special_event_id),
				_ => None,
			},
			created_by: value.created_by,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[TypesenseGenericCollection::new::<RedeemCode>()]
}
