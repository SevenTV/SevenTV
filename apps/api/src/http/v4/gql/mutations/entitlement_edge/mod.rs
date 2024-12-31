use std::sync::Arc;

use async_graphql::Context;
use shared::database::entitlement::EntitlementEdgeKind;
use shared::database::queries::filter;
use shared::database::role::permissions::AdminPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::v4::gql::types::raw_entitlement::EntitlementNodeInput;
use crate::http::v4::gql::types::{EntitlementEdge, EntitlementNodeAny};

mod operation;

#[derive(Default)]
pub struct EntitlementEdgeMutation;

#[async_graphql::Object]
impl EntitlementEdgeMutation {
	#[tracing::instrument(skip_all, name = "EntitlementEdgeMutation::entitlement_edge")]
	#[graphql(guard = "PermissionGuard::one(AdminPermission::ManageEntitlements)")]
	async fn entitlement_edge(
		&self,
		ctx: &Context<'_>,
		from: EntitlementNodeInput,
		to: EntitlementNodeInput,
	) -> Result<operation::EntitlementEdgeOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let from: EntitlementEdgeKind = from.into();
		let to: EntitlementEdgeKind = to.into();

		let edge = shared::database::entitlement::EntitlementEdge::collection(&global.db)
			.find_one(filter::filter! {
				shared::database::entitlement::EntitlementEdge {
					#[query(rename = "_id", flatten)]
					id: shared::database::entitlement::EntitlementEdgeId {
						#[query(serde)]
						from,
						#[query(serde)]
						to,
					},
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to find entitlement edge");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to find entitlement edge")
			})?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "entitlement edge not found"))?;

		Ok(operation::EntitlementEdgeOperation { edge })
	}

	#[tracing::instrument(skip_all, name = "EntitlementEdgeMutation::entitlement_edge")]
	#[graphql(guard = "PermissionGuard::one(AdminPermission::ManageEntitlements)")]
	async fn create(
		&self,
		ctx: &Context<'_>,
		from: EntitlementNodeInput,
		to: EntitlementNodeInput,
	) -> Result<EntitlementEdge<EntitlementNodeAny, EntitlementNodeAny>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let edge = shared::database::entitlement::EntitlementEdge::new(from.into(), to.into(), None);

		shared::database::entitlement::EntitlementEdge::collection(&global.db)
			.insert_one(&edge)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create entitlement edge");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to create entitlement edge")
			})?;

		Ok(EntitlementEdge::from_db(&edge))
	}
}
