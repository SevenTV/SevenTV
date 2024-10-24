use std::sync::Arc;

use async_graphql::{Context, InputObject, Object};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::queries::filter;
use shared::database::queries::filter::Filter;
use shared::database::role::permissions::RolePermission;
use shared::database::role::{Role as DbRole, RoleId};
use shared::database::stored_event::StoredEventRoleData;
use shared::event::{InternalEvent, InternalEventData};
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::role::Role;
use crate::http::v3::validators::NameValidator;
use crate::transactions::{transaction, transaction_with_mutex, GeneralMutexKey, TransactionError};

#[derive(Default)]
pub struct RolesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesMutation {
	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	#[tracing::instrument(skip_all, name = "RolesMutation::create_role")]
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

		let res = transaction::<_, (), _, _>(global, |mut tx| async move {
			let roles = tx
				.find(
					filter::filter! {
						DbRole {}
					},
					FindOptions::builder().sort(doc! { "rank": 1 }).build(),
				)
				.await?;

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

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::Role {
					after: role.clone(),
					data: StoredEventRoleData::Create,
				},
				timestamp: chrono::Utc::now(),
			})?;

			tx.insert_one::<shared::database::role::Role>(&role, None).await?;

			Ok(role)
		})
		.await;

		match res {
			Ok(role) => Ok(Role::from_db(role)),
			Err(e) => {
				tracing::error!(error = %e, "failed to create role");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"failed to create role",
				))
			}
		}
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	#[tracing::instrument(skip_all, name = "RolesMutation::edit_role")]
	async fn edit_role<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		role_id: GqlObjectId,
		data: EditRoleInput,
	) -> Result<Role, ApiError> {
		let _ = (ctx, role_id, data);
		Err(ApiError::not_implemented(
			ApiErrorCode::Unknown,
			"The edit role mutation is not implemented, due to lack of frontend support",
		))
		// let global: &Arc<Global> = ctx
		// 	.data()
		// 	.map_err(|_|
		// ApiError::internal_server_error(ApiErrorCode::MissingContext,
		// "missing global data"))?; let session = ctx
		// 	.data::<Session>()
		// 	.map_err(|_|
		// ApiError::internal_server_error(ApiErrorCode::MissingContext,
		// "missing sesion data"))?; let authed_user = session.user()?;

		// let permissions = match (data.allowed, data.denied) {
		// 	(Some(allowed), Some(denied)) => {
		// 		let allowed: u64 = allowed
		// 			.parse()
		// 			.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest,
		// "invalid allowed permission"))?; 		let allowed =
		// shared::old_types::role_permission::RolePermission::from(allowed);
		// 		let denied: u64 = denied
		// 			.parse()
		// 			.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest,
		// "invalid denied permission"))?; 		let denied =
		// shared::old_types::role_permission::RolePermission::from(denied);

		// 		let role_permissions =
		// 			shared::old_types::role_permission::RolePermission::to_new_permissions(allowed, denied);

		// 		if !authed_user.computed.permissions.is_superset_of(&
		// role_permissions) { 			return Err(ApiError::forbidden(
		// 				ApiErrorCode::LackingPrivileges,
		// 				"the role has a higher permission level than you",
		// 			));
		// 		}

		// 		Some(role_permissions)
		// 	}
		// 	(None, None) => None,
		// 	_ => {
		// 		return Err(ApiError::bad_request(
		// 			ApiErrorCode::LackingPrivileges,
		// 			"must provide both allowed and denied permissions",
		// 		));
		// 	}
		// };

		// let res = transaction_with_mutex(global,
		// Some(GeneralMutexKey::Role(self.id.id()).into()), |mut tx| async move
		// { 	let before_role = tx
		// 		.find_one_and_update(
		// 			filter::filter! {
		// 				DbRole {
		// 					#[query(rename = "_id")]
		// 					id: role_id.id(),
		// 				}
		// 			},
		// 			update::update! {
		// 				#[query(set)]
		// 				DbRole {
		// 					#[query(serde, optional)]
		// 					permissions: permissions.clone(),
		// 					#[query(optional)]
		// 					name: data.name.clone(),
		// 					#[query(optional)]
		// 					color: data.color,
		// 					#[query(optional)]
		// 					rank: data.position.map(|p| p as i32),
		// 					updated_at: chrono::Utc::now(),
		// 					search_updated_at: &None,
		// 				}
		// 			},
		// 			FindOneAndUpdateOptions::builder()
		// 				.return_document(ReturnDocument::Before)
		// 				.build(),
		// 		)
		// 		.await?
		// 		.ok_or_else(|| {
		// 			TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "failed to update role"))
		// 		})?;

		// 	let mut after_role = before_role.clone();

		// 	if let Some(permissions) = &permissions {
		// 		after_role.permissions = permissions.clone();
		// 	}

		// 	if let Some(name) = &data.name {
		// 		after_role.name = name.clone();
		// 	}

		// 	if let Some(color) = data.color {
		// 		after_role.color = Some(color);
		// 	}

		// 	if let Some(position) = data.position {
		// 		after_role.rank = position as i32;
		// 	}

		// 	if permissions.is_some() {
		// 		tx.register_event(InternalEvent {
		// 			actor: Some(authed_user.clone()),
		// 			session_id: session.user_session_id(),
		// 			data: InternalEventData::Role {
		// 				after: after_role.clone(),
		// 				data: StoredEventRoleData::ChangePermissions {
		// 					old: Box::new(before_role.permissions),
		// 					new: Box::new(after_role.permissions.clone()),
		// 				},
		// 			},
		// 			timestamp: chrono::Utc::now(),
		// 		})?;
		// 	}

		// 	if data.name.is_some() {
		// 		tx.register_event(InternalEvent {
		// 			actor: Some(authed_user.clone()),
		// 			session_id: session.user_session_id(),
		// 			data: InternalEventData::Role {
		// 				after: after_role.clone(),
		// 				data: StoredEventRoleData::ChangeName {
		// 					old: before_role.name,
		// 					new: after_role.name.clone(),
		// 				},
		// 			},
		// 			timestamp: chrono::Utc::now(),
		// 		})?;
		// 	}

		// 	if data.color.is_some() {
		// 		tx.register_event(InternalEvent {
		// 			actor: Some(authed_user.clone()),
		// 			session_id: session.user_session_id(),
		// 			data: InternalEventData::Role {
		// 				after: after_role.clone(),
		// 				data: StoredEventRoleData::ChangeColor {
		// 					old: before_role.color,
		// 					new: after_role.color,
		// 				},
		// 			},
		// 			timestamp: chrono::Utc::now(),
		// 		})?;
		// 	}

		// 	if data.position.is_some() {
		// 		tx.register_event(InternalEvent {
		// 			actor: Some(authed_user.clone()),
		// 			session_id: session.user_session_id(),
		// 			data: InternalEventData::Role {
		// 				after: after_role.clone(),
		// 				data: StoredEventRoleData::ChangeRank {
		// 					old: before_role.rank,
		// 					new: after_role.rank,
		// 				},
		// 			},
		// 			timestamp: chrono::Utc::now(),
		// 		})?;
		// 	}

		// 	Ok(after_role)
		// })
		// .await;

		// match res {
		// 	Ok(role) => Ok(Role::from_db(role)),
		// 	Err(TransactionError::Custom(e)) => Err(e),
		// 	Err(e) => {
		// 		tracing::error!(error = %e, "failed to edit role");
		// 		Err(ApiError::internal_server_error(
		// 			ApiErrorCode::TransactionError,
		// 			"failed to edit role",
		// 		))
		// 	}
		// }
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Manage)")]
	#[tracing::instrument(skip_all, name = "RolesMutation::delete_role")]
	async fn delete_role<'ctx>(&self, ctx: &Context<'ctx>, role_id: GqlObjectId) -> Result<String, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::Role(role_id.id()).into()),
			|mut tx| async move {
				let role = tx
					.find_one(
						filter::filter! {
							DbRole {
								#[query(rename = "_id")]
								id: role_id.id(),
							}
						},
						None,
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "role not found"))
					})?;

				if !authed_user.computed.permissions.is_superset_of(&role.permissions) {
					return Err(TransactionError::Custom(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"the role has a higher permission level than you",
					)));
				}

				let res = tx
					.delete_one(
						filter::filter! {
							DbRole {
								#[query(rename = "_id")]
								id: role_id.id(),
							}
						},
						None,
					)
					.await?;

				if res.deleted_count == 0 {
					return Err(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::LoadError,
						"role not found",
					)));
				}

				tx.delete(
					Filter::or([
						filter::filter! {
							EntitlementEdge {
								#[query(rename = "_id", flatten)]
								id: EntitlementEdgeId {
									#[query(serde)]
									from: EntitlementEdgeKind::Role {
										role_id: role_id.id(),
									},
								}
							}
						},
						filter::filter! {
							EntitlementEdge {
								#[query(rename = "_id", flatten)]
								id: EntitlementEdgeId {
									#[query(serde)]
									to: EntitlementEdgeKind::Role {
										role_id: role_id.id(),
									},
								}
							}
						},
					]),
					None,
				)
				.await?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::Role {
						after: role,
						data: StoredEventRoleData::Delete,
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(())
			},
		)
		.await;

		match res {
			Ok(_) => Ok(String::new()),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "failed to delete role");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"failed to delete role",
				))
			}
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
