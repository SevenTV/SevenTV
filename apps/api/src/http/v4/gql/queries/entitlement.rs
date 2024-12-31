use std::sync::Arc;

use async_graphql::Context;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::entitlement_edge::EntitlementEdgeGraphTraverse;
use shared::database::graph::{Direction, GraphTraverse};
use shared::database::role::permissions::AdminPermission;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::v4::gql::types::raw_entitlement::{EntitlementNodeInput, RawEntitlements};

#[derive(Default)]
pub struct EntitlementQuery;

#[async_graphql::Object]
impl EntitlementQuery {
	#[tracing::instrument(skip_all, name = "EntitlementQuery::traverse")]
	#[graphql(guard = "PermissionGuard::one(AdminPermission::ManageEntitlements)")]
	async fn traverse(&self, ctx: &Context<'_>, from: EntitlementNodeInput) -> Result<RawEntitlements, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let traverse = &EntitlementEdgeGraphTraverse {
			inbound_loader: &global.entitlement_edge_inbound_loader,
			outbound_loader: &global.entitlement_edge_outbound_loader,
		};

		let start_node: EntitlementEdgeKind = from.into();

		// follow the graph
		let edges = traverse
			.traversal(Direction::Outbound, [start_node])
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to traverse entitlement edges"))?;

		Ok(RawEntitlements::from_db(&edges))
	}
}
