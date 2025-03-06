use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::user::FullUser;

use crate::http::v4::gql::types::{
	EntitlementEdge, EntitlementNodeAny, EntitlementNodeBadge, EntitlementNodePaint, EntitlementNodeProduct,
};

#[derive(Default, async_graphql::SimpleObject)]
pub struct UserInventory {
	pub paints: Vec<EntitlementEdge<EntitlementNodeAny, EntitlementNodePaint>>,
	pub badges: Vec<EntitlementEdge<EntitlementNodeAny, EntitlementNodeBadge>>,
	pub products: Vec<EntitlementEdge<EntitlementNodeAny, EntitlementNodeProduct>>,
}

impl UserInventory {
	pub fn from_user(user: &FullUser) -> Self {
		let Some(raw_entitlements) = &user.computed.raw_entitlements else {
			return Self::default();
		};

		let mut paints = Vec::new();
		let mut badges = Vec::new();
		let mut products = Vec::new();

		for entitlement in raw_entitlements {
			match entitlement.id.to {
				EntitlementEdgeKind::Paint { paint_id } => paints.push(EntitlementEdge {
					from: EntitlementNodeAny::from_db(&entitlement.id.from),
					to: EntitlementNodePaint { paint_id },
				}),
				EntitlementEdgeKind::Badge { badge_id } => badges.push(EntitlementEdge {
					from: EntitlementNodeAny::from_db(&entitlement.id.from),
					to: EntitlementNodeBadge { badge_id },
				}),
				EntitlementEdgeKind::Product { product_id } => products.push(EntitlementEdge {
					from: EntitlementNodeAny::from_db(&entitlement.id.from),
					to: EntitlementNodeProduct { product_id },
				}),
				_ => {}
			}
		}

		badges.sort_by_key(|b| b.to.badge_id);
		paints.sort_by_key(|p| p.to.paint_id);
		products.sort_by_key(|p| p.to.product_id);

		UserInventory {
			paints,
			badges,
			products,
		}
	}
}
