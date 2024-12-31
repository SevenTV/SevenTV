use std::sync::Arc;

use async_graphql::Context;
use shared::database::cron_job::{CronJob, CronJobId};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::UserPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;

#[derive(Default)]
pub struct JobMutation;

#[async_graphql::Object]
impl JobMutation {
	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageAny)")]
	async fn rerun_subscription_refresh_job(&self, ctx: &Context<'_>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let res = CronJob::collection(&global.db)
			.update_one(
				filter::filter! {
					CronJob {
						#[query(rename = "_id")]
						id: CronJobId::SubscriptionRefresh,
					}
				},
				update::update! {
					#[query(set)]
					CronJob {
						next_run: chrono::Utc::now(),
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					},
				},
			)
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update job"))?;

		Ok(res.modified_count > 0)
	}
}
