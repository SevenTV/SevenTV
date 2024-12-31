use std::sync::Arc;

use async_graphql::Context;
use mongodb::options::FindOneAndUpdateOptions;
use shared::database::queries::{filter, update};
use shared::database::role::permissions::AdminPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::v4::gql::types::RedeemCode;

pub struct RedeemCodeOperation {
	pub code: shared::database::product::codes::RedeemCode,
}

#[async_graphql::Object]
impl RedeemCodeOperation {
	#[tracing::instrument(skip_all, name = "RedeemCodeOperation::deactivate")]
	#[graphql(guard = "PermissionGuard::one(AdminPermission::ManageRedeemCodes)")]
	async fn deactivate(&self, ctx: &Context<'_>) -> Result<RedeemCode, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let code = shared::database::product::codes::RedeemCode::collection(&global.db)
			.find_one_and_update(
				filter::filter! {
					shared::database::product::codes::RedeemCode {
						#[query(rename = "_id")]
						id: self.code.id,
					}
				},
				update::update! {
					#[query(set)]
					shared::database::product::codes::RedeemCode {
						remaining_uses: 0,
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					}
				},
			)
			.with_options(
				FindOneAndUpdateOptions::builder()
					.return_document(mongodb::options::ReturnDocument::After)
					.build(),
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update code");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to update code")
			})?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "code not found"))?;

		Ok(code.into())
	}
}
