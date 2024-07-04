use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::bson::{doc, to_bson};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogId, AuditLogUserData};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::role::permissions::{AdminPermission, PermissionsExt, RolePermission, UserPermission};
use shared::database::user::User;
use shared::database::Collection;
use shared::old_types::cosmetic::CosmeticKind;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::user::{UserConnection, UserEditor};
use crate::http::v3::gql::types::ListItemAction;
use crate::http::v3::types::UserEditorModelPermission;

#[derive(Default)]
pub struct UsersMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl UsersMutation {
	async fn user(&self, id: GqlObjectId) -> UserOps {
		UserOps { id }
	}
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserOps {
	id: GqlObjectId,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl UserOps {
	async fn connections<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: String,
		data: UserConnectionUpdate,
	) -> Result<Option<Vec<Option<UserConnection>>>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await?;

		if !(auth_session.user_id() == self.id.id() || user.has(UserPermission::ManageAny)) {
			return Err(ApiError::FORBIDDEN);
		}

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let mut update = doc! {};
		let mut update_pull = doc! {};

		if let Some(emote_set_id) = data.emote_set_id {
			// check if set exists
			global
				.emote_set_by_id_loader()
				.load(emote_set_id.id())
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::NOT_FOUND)?;

			update.insert("style.active_emote_set_id", emote_set_id.0);
		}

		if let Some(true) = data.unlink {
			update_pull.insert("connections", doc! { "platform_id": id });
		}

		let Some(user) = User::collection(global.db())
			.find_one_and_update(
				doc! {
					"_id": self.id.0,
				},
				doc! {
					"$set": update,
					"$pull": update_pull,
				},
			)
			.with_options(
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.build(),
			)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to update user");
				ApiError::INTERNAL_SERVER_ERROR
			})?
		else {
			return Ok(None);
		};

		Ok(Some(
			user.connections
				.into_iter()
				.map(|c| Some(UserConnection::from_db(c, &user.style, &global_config)))
				.collect(),
		))
	}

	async fn editors(
		&self,
		ctx: &Context<'_>,
		editor_id: GqlObjectId,
		data: UserEditorUpdate,
	) -> Result<Option<Vec<Option<UserEditor>>>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await?;

		if !(auth_session.user_id() == self.id.id() || user.has(UserPermission::ManageAny)) {
			return Err(ApiError::FORBIDDEN);
		}

		let mut session = global.mongo().start_session().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		if let Some(permissions) = data.permissions {
			if permissions == UserEditorModelPermission::none() {
				// Remove editor
				let res = shared::database::user::editor::UserEditor::collection(global.db())
					.delete_one(doc! {
						"user_id": self.id.0,
						"editor_id": editor_id.0,
					})
					.session(&mut session)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to delete editor");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				if res.deleted_count > 0 {
					AuditLog::collection(global.db())
						.insert_one(AuditLog {
							id: AuditLogId::new(),
							actor_id: Some(auth_session.user_id()),
							data: AuditLogData::User {
								target_id: self.id.id(),
								data: AuditLogUserData::RemoveEditor {
									editor_id: editor_id.id(),
								},
							},
						})
						.session(&mut session)
						.await
						.map_err(|err| {
							tracing::error!(error = %err, "failed to insert audit log");
							ApiError::INTERNAL_SERVER_ERROR
						})?;
				}
			} else {
				// Add or update editor
				let res = shared::database::user::editor::UserEditor::collection(global.db())
					.update_one(
						doc! {
							"user_id": self.id.0,
							"editor_id": editor_id.0,
						},
						doc! {
							"$set": {
								"permissions": to_bson(&permissions.to_db()).map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?,
							},
						},
					)
					.upsert(user.has(UserPermission::InviteEditors))
					.session(&mut session)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to update editor");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				// inserted
				if res.matched_count == 0 {
					AuditLog::collection(global.db())
						.insert_one(AuditLog {
							id: AuditLogId::new(),
							actor_id: Some(auth_session.user_id()),
							data: AuditLogData::User {
								target_id: self.id.id(),
								data: AuditLogUserData::AddEditor {
									editor_id: editor_id.id(),
								},
							},
						})
						.session(&mut session)
						.await
						.map_err(|err| {
							tracing::error!(error = %err, "failed to insert audit log");
							ApiError::INTERNAL_SERVER_ERROR
						})?;
				}
			}
		}

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let editors = global
			.user_editor_by_user_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(Some(
			editors
				.into_iter()
				.filter_map(|e| UserEditor::from_db(e, false))
				.map(Some)
				.collect(),
		))
	}

	async fn cosmetics<'ctx>(&self, ctx: &Context<'ctx>, update: UserCosmeticUpdate) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let authed_user = auth_session.user(global).await?;

		if !(auth_session.user_id() == self.id.id() || authed_user.has(UserPermission::ManageAny)) {
			return Err(ApiError::FORBIDDEN);
		}

		if !update.selected {
			return Ok(true);
		}

		let user = global
			.user_loader()
			.load(global, self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		match update.kind {
			CosmeticKind::Paint => {
				// check if user has badge
				if !user.computed.entitlements.paints.contains(&update.id.id()) {
					return Err(ApiError::FORBIDDEN);
				}

				let res = User::collection(global.db())
					.update_one(
						doc! {
							"_id": self.id.0,
						},
						doc! {
							"$set": {
								"style.active_paint_id": update.id.0,
							}
						},
					)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to update user");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				Ok(res.modified_count == 1)
			}
			CosmeticKind::Badge => {
				// check if user has paint
				if !user.computed.entitlements.badges.contains(&update.id.id()) {
					return Err(ApiError::FORBIDDEN);
				}

				let res = User::collection(global.db())
					.update_one(
						doc! {
							"_id": self.id.0,
						},
						doc! {
							"$set": {
								"style.active_badge_id": update.id.0,
							}
						},
					)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to update user");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				Ok(res.modified_count == 1)
			}
			CosmeticKind::Avatar => Err(ApiError::NOT_IMPLEMENTED),
		}
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Assign)")]
	async fn roles<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		role_id: GqlObjectId,
		action: ListItemAction,
	) -> Result<Vec<GqlObjectId>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await?;

		let role = global
			.role_by_id_loader()
			.load(role_id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		if role.permissions > user.computed.permissions && !user.has(AdminPermission::SuperAdmin) {
			return Err(ApiError::FORBIDDEN);
		}

		let mut session = global.mongo().start_session().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let roles = match action {
			ListItemAction::Add => {
				if user.computed.entitlements.roles.contains(&role_id.id()) {
					return Ok(user.computed.entitlements.roles.iter().copied().map(Into::into).collect());
				}

				let edge = EntitlementEdge {
					id: EntitlementEdgeId {
						from: EntitlementEdgeKind::User { user_id: self.id.id() },
						to: EntitlementEdgeKind::Role { role_id: role_id.id() },
						managed_by: None,
					},
				};

				EntitlementEdge::collection(global.db())
					.insert_one(&edge)
					.session(&mut session)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to insert entitlement edge");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				AuditLog::collection(global.db())
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(auth_session.user_id()),
						data: AuditLogData::User {
							target_id: self.id.id(),
							data: AuditLogUserData::AddRole { role_id: role_id.id() },
						},
					})
					.session(&mut session)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				user.computed
					.entitlements
					.roles
					.iter()
					.copied()
					.chain(std::iter::once(role_id.id()))
					.map(Into::into)
					.collect()
			}
			ListItemAction::Remove => {
				let from = EntitlementEdgeKind::User { user_id: self.id.id() };
				let to = EntitlementEdgeKind::Role { role_id: role_id.id() };

				let res = EntitlementEdge::collection(global.db())
					.delete_one(doc! {
						"_id.from": to_bson(&from).expect("failed to convert to bson"),
						"_id.to": to_bson(&to).expect("failed to convert to bson"),
					})
					.session(&mut session)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to delete entitlement edge");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				if res.deleted_count > 0 {
					AuditLog::collection(global.db())
						.insert_one(AuditLog {
							id: AuditLogId::new(),
							actor_id: Some(auth_session.user_id()),
							data: AuditLogData::User {
								target_id: self.id.id(),
								data: AuditLogUserData::RemoveRole { role_id: role_id.id() },
							},
						})
						.session(&mut session)
						.await
						.map_err(|err| {
							tracing::error!(error = %err, "failed to insert audit log");
							ApiError::INTERNAL_SERVER_ERROR
						})?;

					user.computed
						.entitlements
						.roles
						.iter()
						.copied()
						.filter(|id| *id != role_id.id())
						.map(Into::into)
						.collect()
				} else {
					user.computed.entitlements.roles.iter().copied().map(Into::into).collect()
				}
			}
			ListItemAction::Update => return Err(ApiError::NOT_IMPLEMENTED),
		};

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(roles)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserConnectionUpdate {
	emote_set_id: Option<GqlObjectId>,
	unlink: Option<bool>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserEditorUpdate {
	permissions: Option<UserEditorModelPermission>,
	visible: Option<bool>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserCosmeticUpdate {
	id: GqlObjectId,
	kind: CosmeticKind,
	selected: bool,
}
