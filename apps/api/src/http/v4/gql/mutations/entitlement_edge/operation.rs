use std::sync::Arc;

use async_graphql::Context;
use shared::database::queries::filter;
use shared::database::role::permissions::AdminPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;

pub struct EntitlementEdgeOperation {
	pub edge: shared::database::entitlement::EntitlementEdge,
}

#[async_graphql::Object]
impl EntitlementEdgeOperation {
	#[tracing::instrument(skip_all, name = "EntitlementEdgeOperation::delete")]
	#[graphql(guard = "PermissionGuard::one(AdminPermission::ManageEntitlements)")]
	async fn delete(&self, ctx: &Context<'_>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let res = shared::database::entitlement::EntitlementEdge::collection(&global.db)
			.delete_one(filter::filter! {
				shared::database::entitlement::EntitlementEdge {
					#[query(rename = "_id", serde)]
					id: &self.edge.id,
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to delete entitlement edge");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to delete entitlement edge")
			})?;

		Ok(res.deleted_count == 1)
	}
}
