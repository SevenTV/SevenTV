use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::Context;
use futures::{TryFutureExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use shared::database::queries::filter;
use shared::database::role::permissions::RolePermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::v4::gql::types::Role;

#[derive(Default)]
pub struct RoleQuery;

#[async_graphql::Object]
impl RoleQuery {
	#[tracing::instrument(skip_all, name = "RoleQuery::roles")]
	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let roles: Vec<_> = shared::database::role::Role::collection(&global.db)
			.find(filter::filter!(shared::database::role::Role {}))
			.with_options(FindOptions::builder().sort(doc! { "rank": -1 }).build())
			.into_future()
			.and_then(|f| f.try_collect::<Vec<shared::database::role::Role>>())
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to query roles");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to query roles")
			})?;

		Ok(roles.into_iter().map(Into::into).collect())
	}
}
