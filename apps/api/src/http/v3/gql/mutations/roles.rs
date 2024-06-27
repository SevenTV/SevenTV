use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::global::GlobalConfig;
use shared::database::role::RoleId;
use shared::database::Collection;
use shared::database::role::permissions::RolePermission;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::Role;

#[derive(Default)]
pub struct RolesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesMutation {
	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn create_role<'ctx>(&self, ctx: &Context<'ctx>, data: CreateRoleInput) -> Result<Role, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let allowed: u64 = data.allowed.parse().map_err(|_| ApiError::BAD_REQUEST)?;
		let allowed = shared::old_types::role_permission::RolePermission::from(allowed);
		let denied: u64 = data.denied.parse().map_err(|_| ApiError::BAD_REQUEST)?;
		let denied = shared::old_types::role_permission::RolePermission::from(denied);

		let role = shared::database::role::Role {
			id: RoleId::new(),
			name: data.name,
			description: None,
			tags: vec![],
			permissions: shared::old_types::role_permission::RolePermission::to_new_permissions(allowed, denied),
			hoist: false,
			color: Some(data.color),
		};

		let mut session = global.mongo().start_session(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let global_config = GlobalConfig::collection(global.db())
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

		shared::database::role::Role::collection(global.db())
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

	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn edit_role<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		role_id: GqlObjectId,
		data: EditRoleInput,
	) -> Result<Role, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut session = global.mongo().start_session(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let mut role_ids_push = doc! {
			"$each": [role_id.0],
		};

		if let Some(position) = data.position {
			role_ids_push.insert("$position", position);
		}

		let global_config = GlobalConfig::collection(global.db())
			.find_one_and_update_with_session(
				doc! {},
				vec![
					doc! {
						"$pull": {
							"role_ids": role_id.0,
						}
					},
					doc! {
						"$push": {
							"role_ids": role_ids_push,
						}
					},
				],
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

		let mut update = doc! {};

		if let Some(name) = data.name {
			update.insert("name", name);
		}

		if let Some(color) = data.color {
			update.insert("color", color);
		}

		match (data.allowed, data.denied) {
			(Some(allowed), Some(denied)) => {
				let allowed: u64 = allowed.parse().map_err(|_| ApiError::BAD_REQUEST)?;
				let allowed = shared::old_types::role_permission::RolePermission::from(allowed);
				let denied: u64 = denied.parse().map_err(|_| ApiError::BAD_REQUEST)?;
				let denied = shared::old_types::role_permission::RolePermission::from(denied);

				update.insert(
					"permissions",
					to_bson(&shared::old_types::role_permission::RolePermission::to_new_permissions(allowed, denied))
						.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?,
				);
			}
			(None, None) => {}
			_ => {
				return Err(ApiError::new_const(
					StatusCode::BAD_REQUEST,
					"must provide both allowed and denied permissions",
				));
			}
		}

		let role = shared::database::role::Role::collection(global.db())
			.find_one_and_update_with_session(
				doc! {
					"_id": role_id.0,
				},
				doc! {
					"$set": update,
				},
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.build(),
				&mut session,
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update role");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(Role::from_db(role, &global_config))
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn delete_role<'ctx>(&self, ctx: &Context<'ctx>, role_id: GqlObjectId) -> Result<String, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let res = shared::database::role::Role::collection(global.db())
			.delete_one(
				doc! {
					"_id": role_id.0,
				},
				None,
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to delete role");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		if res.deleted_count == 1 {
			Ok(String::new())
		} else {
			Err(ApiError::NOT_FOUND)
		}
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateRoleInput {
	name: String,
	color: i32,
	allowed: String,
	denied: String,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditRoleInput {
	name: Option<String>,
	color: Option<i32>,
	allowed: Option<String>,
	denied: Option<String>,
	position: Option<u32>,
}
