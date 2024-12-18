use itertools::Itertools;

use super::{EntitlementEdge, EntitlementNodeAny};

#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct RawEntitlements {
	pub edges: Vec<EntitlementEdge<EntitlementNodeAny, EntitlementNodeAny>>,
}

impl RawEntitlements {
	pub fn from_db(edges: &[shared::database::entitlement::EntitlementEdge]) -> Self {
		Self {
			edges: edges.iter().unique().map(EntitlementEdge::from_db).collect(),
		}
	}
}

#[async_graphql::ComplexObject]
impl RawEntitlements {
	async fn nodes(&self) -> Vec<&EntitlementNodeAny> {
		self.edges.iter().flat_map(|edge| [&edge.from, &edge.to]).unique().collect()
	}
}
