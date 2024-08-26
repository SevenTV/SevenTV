pub mod codes;
pub mod invoice;
pub mod promotion;
pub mod subscription;

use chrono::Utc;

use super::TypesenseGenericCollection;
use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString};
use crate::database::product::ProductId;
use crate::database::{self};
use crate::typesense::types::TypesenseCollection;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypesenseCollection)]
#[typesense(collection_name = "products")]
#[serde(deny_unknown_fields)]
pub struct Product {
	pub id: ProductId,
	pub name: String,
	pub description: Option<String>,
	pub default_currency: stripe::Currency,
	pub default_price: i32,
	pub currencies: Vec<stripe::Currency>,
	/// The `to` field of the `EntitlementEdge` where the `from` field is this
	/// product.
	pub entitlement_grants: Vec<EntitlementEdgeKindString>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl Product {
	pub fn from_db(
		value: database::product::Product,
		entitlement_grants: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			entitlement_grants: entitlement_grants.into_iter().map(Into::into).collect(),
			default_currency: value.default_currency,
			default_price: value.currency_prices.get(&value.default_currency).copied().unwrap_or(0),
			currencies: value.currency_prices.keys().copied().collect(),
			created_at: value.created_at.timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	std::iter::once(TypesenseGenericCollection::new::<Product>())
		.chain(codes::typesense_collections())
		.chain(invoice::typesense_collections())
		.chain(promotion::typesense_collections())
		.chain(subscription::typesense_collections())
}
