use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::Context;
use futures::{TryFutureExt, TryStreamExt};
use mongodb::options::ReadPreference;
use shared::database::queries::filter;
use shared::database::role::permissions::UserPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::v4::gql::types::SpecialEvent;

#[derive(Default)]
pub struct SpecialEventQuery;

#[async_graphql::Object]
impl SpecialEventQuery {
	#[tracing::instrument(skip_all, name = "SpecialEventQuery::special_events")]
	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageBilling)")]
	async fn special_events(&self, ctx: &Context<'_>) -> Result<Vec<SpecialEvent>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let events: Vec<_> = shared::database::product::special_event::SpecialEvent::collection(&global.db)
			.find(filter::filter! {
				shared::database::product::special_event::SpecialEvent {}
			})
			.batch_size(1000)
			.selection_criteria(ReadPreference::SecondaryPreferred { options: None }.into())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to load special events");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load special events")
			})?;

		Ok(events.into_iter().map(Into::into).collect())
	}
}
