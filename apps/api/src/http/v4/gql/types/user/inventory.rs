use std::sync::Arc;
use crate::global::Global;
use crate::http::v4::gql::types::raw_entitlement::EntitlementNodeInput;
use crate::http::v4::gql::types::raw_entitlement::EntitlementNodeTypeInput;
use crate::http::v4::gql::types::{EntitlementNodeAny, EntitlementNodeBadge, EntitlementNodePaint, EntitlementNodeProduct, User};
use crate::http::ApiError;
use crate::http::ApiErrorCode;
use async_graphql::OutputType;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeKind};
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::Direction;
use shared::database::graph::GraphTraverse;

#[derive(async_graphql::SimpleObject)]
#[graphql(concrete(name = "InventoryEntitlementEdgeAnyPaint", params(EntitlementNodeAny, EntitlementNodePaint)))]
#[graphql(concrete(name = "InventoryEntitlementEdgeAnyBadge", params(EntitlementNodeAny, EntitlementNodeBadge)))]
#[graphql(concrete(name = "InventoryEntitlementEdgeAnyProduct", params(EntitlementNodeAny, EntitlementNodeProduct)))]
pub struct InventoryEntitlementEdge<From: OutputType, To: OutputType> {
	pub from: From,
	pub to: To,
	/// Whether this entitlement is directly connected to the user
	pub accessible: bool,
}

#[derive(Default, async_graphql::SimpleObject)]
pub struct UserInventory {
	pub paints: Vec<InventoryEntitlementEdge<EntitlementNodeAny, EntitlementNodePaint>>,
	pub badges: Vec<InventoryEntitlementEdge<EntitlementNodeAny, EntitlementNodeBadge>>,
	pub products: Vec<InventoryEntitlementEdge<EntitlementNodeAny, EntitlementNodeProduct>>,
}

impl UserInventory {
	pub async fn from_user(user: &User, include_inaccessible: bool, global: &Arc<Global>) -> Result<Self, ApiError> {
		let id = user.id;
		let user_node = EntitlementEdgeKind::User { user_id: id };
		let sub_node = EntitlementNodeInput {
			ty: EntitlementNodeTypeInput::Subscription,
			id: id.cast(),
		}
		.into();

		let traverse = &EntitlementEdgeGraphTraverse {
			inbound_loader: &global.entitlement_edge_inbound_loader,
			outbound_loader: &global.entitlement_edge_outbound_loader,
		};

		let mut direct = fnv::FnvHashSet::default();
		let mut found_subscription = false;

		let connected_edges = if user.full_user.all_cosmetics {
			found_subscription = true;
			global
				.user_loader
				.load_user(global, user.full_user.user.clone())
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
				.computed.raw_entitlements.unwrap_or_default()
		} else {
			traverse
				.traversal_filter(Direction::Outbound, [user_node], |kind| {
					if kind == &sub_node {
						found_subscription = true;
					}
					direct.insert(kind.clone())
				})
				.await
				.map_err(|_| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to traverse entitlement edges")
				})?
		};

		let disconnected_edges = if include_inaccessible && !found_subscription {
			traverse
				.traversal_filter(Direction::Outbound, [sub_node], |kind| !direct.contains(kind))
				.await
				.map_err(|_| {
					ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to traverse entitlement edges")
				})?
		} else {
			vec![]
		};

		let mut paints = Vec::new();
		let mut badges = Vec::new();
		let mut products = Vec::new();

		let mut add_entitlement = |edge: EntitlementEdge, accessible: bool| match edge.id.to {
			EntitlementEdgeKind::Paint { paint_id } => paints.push(InventoryEntitlementEdge {
				from: EntitlementNodeAny::from_db(&edge.id.from),
				to: EntitlementNodePaint { paint_id },
				accessible,
			}),
			EntitlementEdgeKind::Badge { badge_id } => badges.push(InventoryEntitlementEdge {
				from: EntitlementNodeAny::from_db(&edge.id.from),
				to: EntitlementNodeBadge { badge_id },
				accessible,
			}),
			EntitlementEdgeKind::Product { product_id } => products.push(InventoryEntitlementEdge {
				from: EntitlementNodeAny::from_db(&edge.id.from),
				to: EntitlementNodeProduct { product_id },
				accessible,
			}),
			_ => {}
		};

		for edge in connected_edges {
			add_entitlement(edge, true);
		}

		for edge in disconnected_edges {
			add_entitlement(edge, false);
		}

		badges.sort_by_key(|b| b.to.badge_id);
		paints.sort_by_key(|p| p.to.paint_id);
		products.sort_by_key(|p| p.to.product_id);

		Ok(UserInventory {
			paints,
			badges,
			products,
		})
	}
}