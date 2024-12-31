use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::Context;
use futures::{TryFutureExt, TryStreamExt};
use shared::database::queries::filter;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v4::gql::types::Paint;

#[derive(Default)]
pub struct PaintQuery;

#[async_graphql::Object]
impl PaintQuery {
	#[tracing::instrument(skip_all, name = "PaintQuery::paints")]
	async fn paints(&self, ctx: &Context<'_>) -> Result<Vec<Paint>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let paints: Vec<_> = shared::database::paint::Paint::collection(&global.db)
			.find(filter::filter!(shared::database::paint::Paint {}))
			.into_future()
			.and_then(|f| f.try_collect::<Vec<shared::database::paint::Paint>>())
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to query paints");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to query paints")
			})?;

		Ok(paints
			.into_iter()
			.map(|p| Paint::from_db(p, &global.config.api.cdn_origin))
			.collect())
	}
}
