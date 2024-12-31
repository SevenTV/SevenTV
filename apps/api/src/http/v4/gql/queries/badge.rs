use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::Context;
use futures::{TryFutureExt, TryStreamExt};
use shared::database::queries::filter;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::Badge;

#[derive(Default)]
pub struct BadgeQuery;

#[async_graphql::Object]
impl BadgeQuery {
	#[tracing::instrument(skip_all, name = "BadgeQuery::badges")]
	async fn badges(&self, ctx: &Context<'_>) -> Result<Vec<Badge>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let badges: Vec<_> = shared::database::badge::Badge::collection(&global.db)
			.find(filter::filter!(shared::database::badge::Badge {}))
			.into_future()
			.and_then(|f| f.try_collect::<Vec<shared::database::badge::Badge>>())
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to query badges");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to query badges")
			})?;

		Ok(badges
			.into_iter()
			.map(|b| Badge::from_db(b, &global.config.api.cdn_origin))
			.collect())
	}
}
