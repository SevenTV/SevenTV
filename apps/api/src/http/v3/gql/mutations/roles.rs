use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::{self, Collection, RolePermission};
use shared::old_types::RoleObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::Role;

#[derive(Default)]
pub struct RolesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesMutation {
	#[graphql(guard = "PermissionGuard::one(RolePermission::Create)")]
	async fn create_role<'ctx>(&self, ctx: &Context<'ctx>, data: CreateRoleInput) -> Result<Role, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let allowed: u64 = data.allowed.parse().map_err(|_| ApiError::BAD_REQUEST)?;
		let allowed = shared::old_types::RolePermission::from(allowed);
		let denied: u64 = data.denied.parse().map_err(|_| ApiError::BAD_REQUEST)?;
		let denied = shared::old_types::RolePermission::from(denied);

		let role = database::Role {
			name: data.name,
			permissions: shared::old_types::RolePermission::to_new_permissions(allowed, denied),
			color: data.color,
			..Default::default()
		};

		let mut session = global.mongo().start_session(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let global_config = database::GlobalConfig::collection(global.db())
			.find_one_and_update_with_session(
				doc! {},
				doc! {
					"$push": {
						"role_ids": role.id,
					}
				},
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.build(),
				&mut session,
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update global config");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		database::Role::collection(global.db())
			.insert_one_with_session(&role, None, &mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert role");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(Role::from_db(role, &global_config))
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Edit)")]
	async fn edit_role(&self, role_id: RoleObjectId, data: EditRoleInput) -> Result<Role, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Delete)")]
	async fn delete_role(&self, role_id: RoleObjectId) -> Result<String, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateRoleInput {
	name: String,
	color: u32,
	allowed: String,
	denied: String,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditRoleInput {
	name: Option<String>,
	color: Option<u32>,
	allowed: Option<String>,
	denied: Option<String>,
	position: Option<u32>,
}
