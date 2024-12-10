use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::user::FullUser;

use crate::http::v4::gql::types::{EntitlementEdge, EntitlementNodeAny, EntitlementNodeBadge, EntitlementNodePaint};

#[derive(Default, async_graphql::SimpleObject)]
pub struct UserInventory {
	pub paints: Vec<EntitlementEdge<EntitlementNodeAny, EntitlementNodePaint>>,
	pub badges: Vec<EntitlementEdge<EntitlementNodeAny, EntitlementNodeBadge>>,
}

impl UserInventory {
	pub fn from_user(user: &FullUser) -> Self {
		let Some(raw_entitlements) = &user.computed.raw_entitlements else {
			return Self::default();
		};

		let mut paints = Vec::new();
		let mut badges = Vec::new();

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
				_ => {}
			}
		}

		badges.sort_by_key(|b| b.to.badge_id);
		paints.sort_by_key(|p| p.to.paint_id);

		UserInventory { paints, badges }
	}
}
