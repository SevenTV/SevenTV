use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use futures::{TryFutureExt, TryStreamExt};
use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument};
use shared::database::role::permissions::{AdminPermission, PermissionsExt, RolePermission};
use shared::database::role::RoleId;
use shared::database::MongoCollection;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::role::Role;

#[derive(Default)]
pub struct RolesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesMutation {
	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn create_role<'ctx>(&self, ctx: &Context<'ctx>, data: CreateRoleInput) -> Result<Role, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await.map_err(|_| ApiError::UNAUTHORIZED)?;

		let allowed: u64 = data.allowed.parse().map_err(|_| ApiError::BAD_REQUEST)?;
		let allowed = shared::old_types::role_permission::RolePermission::from(allowed);
		let denied: u64 = data.denied.parse().map_err(|_| ApiError::BAD_REQUEST)?;
		let denied = shared::old_types::role_permission::RolePermission::from(denied);

		let role_permissions = shared::old_types::role_permission::RolePermission::to_new_permissions(allowed, denied);

		if role_permissions > user.computed.permissions && !user.has(AdminPermission::SuperAdmin) {
			return Err(ApiError::FORBIDDEN);
		}

		let roles: Vec<shared::database::role::Role> = shared::database::role::Role::collection(global.db())
			.find(doc! {})
			.with_options(FindOptions::builder().sort(doc! { "rank": 1 }).build())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let mut rank = 0;

		while roles.iter().any(|r| r.rank == rank) {
			rank += 1;
		}

		let role = shared::database::role::Role {
			id: RoleId::new(),
			name: data.name,
			description: None,
			tags: vec![],
			permissions: role_permissions,
			hoist: false,
			color: Some(data.color),
			rank,
			created_by: user.id,
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
			applied_rank: None,
		};

		shared::database::role::Role::collection(global.db())
			.insert_one(&role)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert role");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(Role::from_db(role))
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn edit_role<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		role_id: GqlObjectId,
		data: EditRoleInput,
	) -> Result<Role, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await.map_err(|_| ApiError::UNAUTHORIZED)?;

		let mut session = global.mongo().start_session().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

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

				let role_permissions =
					shared::old_types::role_permission::RolePermission::to_new_permissions(allowed, denied);

				if role_permissions > user.computed.permissions && !user.has(AdminPermission::SuperAdmin) {
					return Err(ApiError::FORBIDDEN);
				}

				update.insert(
					"permissions",
					to_bson(&role_permissions).map_err(|err| {
						tracing::error!(error = %err, "failed to serialize role permissions");
						ApiError::INTERNAL_SERVER_ERROR
					})?,
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

		update.insert("updated_at", Some(bson::DateTime::from(chrono::Utc::now())));

		let role = shared::database::role::Role::collection(global.db())
			.find_one_and_update(
				doc! {
					"_id": role_id.0,
				},
				doc! {
					"$set": update,
				},
			)
			.with_options(
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.build(),
			)
			.session(&mut session)
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

		Ok(Role::from_db(role))
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn delete_role<'ctx>(&self, ctx: &Context<'ctx>, role_id: GqlObjectId) -> Result<String, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await.map_err(|_| ApiError::UNAUTHORIZED)?;

		let role = global
			.role_by_id_loader()
			.load(role_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		if role.permissions > user.computed.permissions && !user.has(AdminPermission::SuperAdmin) {
			return Err(ApiError::FORBIDDEN);
		}

		let res = shared::database::role::Role::collection(global.db())
			.delete_one(doc! {
				"_id": role_id.0,
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to delete role");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		// TODO: remove entitlement edges

		if res.deleted_count > 0 {
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
