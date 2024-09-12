use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use futures::{TryFutureExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::RolePermission;
use shared::database::role::{Role as DbRole, RoleId};
use shared::database::MongoCollection;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::role::Role;
use crate::http::v3::validators::NameValidator;

#[derive(Default)]
pub struct RolesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesMutation {
	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn create_role<'ctx>(&self, ctx: &Context<'ctx>, data: CreateRoleInput) -> Result<Role, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let allowed: u64 = data
			.allowed
			.parse()
			.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid allowed permission"))?;
		let allowed = shared::old_types::role_permission::RolePermission::from(allowed);
		let denied: u64 = data
			.denied
			.parse()
			.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid denied permission"))?;
		let denied = shared::old_types::role_permission::RolePermission::from(denied);

		let role_permissions = shared::old_types::role_permission::RolePermission::to_new_permissions(allowed, denied);

		if !authed_user.computed.permissions.is_superset_of(&role_permissions) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"the role has a higher permission level than you",
			));
		}

		// TODO: events, and this should be in a transaction
		let roles: Vec<shared::database::role::Role> = shared::database::role::Role::collection(&global.db)
			.find(filter::filter! {
				DbRole {}
			})
			.with_options(FindOptions::builder().sort(doc! { "rank": 1 }).build())
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|err| {
				tracing::error!("failed to load: {err}");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load roles")
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
			created_by: authed_user.id,
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
			applied_rank: None,
		};

		shared::database::role::Role::collection(&global.db)
			.insert_one(&role)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert role");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to insert role")
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
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let permissions = match (data.allowed, data.denied) {
			(Some(allowed), Some(denied)) => {
				let allowed: u64 = allowed
					.parse()
					.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid allowed permission"))?;
				let allowed = shared::old_types::role_permission::RolePermission::from(allowed);
				let denied: u64 = denied
					.parse()
					.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid denied permission"))?;
				let denied = shared::old_types::role_permission::RolePermission::from(denied);

				let role_permissions =
					shared::old_types::role_permission::RolePermission::to_new_permissions(allowed, denied);

				if !authed_user.computed.permissions.is_superset_of(&role_permissions) {
					return Err(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"the role has a higher permission level than you",
					));
				}

				Some(role_permissions)
			}
			(None, None) => None,
			_ => {
				return Err(ApiError::bad_request(
					ApiErrorCode::LackingPrivileges,
					"must provide both allowed and denied permissions",
				));
			}
		};

		// TODO: events
		let role = shared::database::role::Role::collection(&global.db)
			.find_one_and_update(
				filter::filter! {
					DbRole {
						#[query(rename = "_id")]
						id: role_id.id(),
					}
				},
				update::update! {
					#[query(set)]
					DbRole {
						#[query(serde, optional)]
						permissions,
						#[query(optional)]
						name: data.name,
						#[query(optional)]
						color: data.color,
						#[query(optional)]
						rank: data.position.map(|p| p as i32),
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.with_options(
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.build(),
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update role");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to update role")
			})?
			.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to update role"))?;

		Ok(Role::from_db(role))
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	async fn delete_role<'ctx>(&self, ctx: &Context<'ctx>, role_id: GqlObjectId) -> Result<String, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let role = global
			.role_by_id_loader
			.load(role_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load role"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "role not found"))?;

		if !authed_user.computed.permissions.is_superset_of(&role.permissions) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"the role has a higher permission level than you",
			));
		}

		// TODO: events
		let res = shared::database::role::Role::collection(&global.db)
			.delete_one(filter::filter! {
				DbRole {
					#[query(rename = "_id")]
					id: role_id.id(),
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to delete role");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to delete role")
			})?;

		// TODO: remove entitlement edges

		if res.deleted_count > 0 {
			Ok(String::new())
		} else {
			Err(ApiError::not_found(ApiErrorCode::LoadError, "role not found"))
		}
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateRoleInput {
	#[graphql(validator(custom = "NameValidator"))]
	name: String,
	color: i32,
	allowed: String,
	denied: String,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditRoleInput {
	#[graphql(validator(custom = "NameValidator"))]
	name: Option<String>,
	color: Option<i32>,
	allowed: Option<String>,
	denied: Option<String>,
	position: Option<u32>,
}
