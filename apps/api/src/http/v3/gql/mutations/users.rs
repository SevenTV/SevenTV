use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::bson::{doc, to_bson};
use shared::database::role::permissions::UserPermission;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::{UserConnection, UserEditor};
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
	#[graphql(guard = "PermissionGuard::one(UserPermission::Edit)")]
	async fn connections<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: String,
		data: UserConnectionUpdate,
	) -> Result<Option<Vec<Option<UserConnection>>>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let (_, authed_user_perms) = auth_session.user(global).await?;

		if !(auth_session.user_id() == self.id.id() || authed_user_perms.has(UserPermission::Admin)) {
			return Err(ApiError::FORBIDDEN);
		}

		let (_, perms) = load_user_and_permissions(global, self.id.id())
			.await?
			.ok_or(ApiError::NOT_FOUND)?;

		let mut session = global.mongo().start_session(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		if let Some(emote_set_id) = data.emote_set_id {
			// check if set exists
			global.emote_set_by_id_loader()
				.load(emote_set_id.id())
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::NOT_FOUND)?;

			database::User::collection(global.db())
				.update_one_with_session(
					doc! {
						"_id": self.id.id(),
					},
					doc! {
						"$set": {
							"active_emote_set_ids": vec![emote_set_id.id()],
						},
					},
					None,
					&mut session,
				)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to update user");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		}

		if let Some(true) = data.unlink {
			database::UserConnection::collection(global.db())
				.delete_one_with_session(
					doc! {
						"platform_id": id,
					},
					None,
					&mut session,
				)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to delete connection");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		}

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let connections = global
			.user_connection_by_user_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		let slots = perms.emote_set_slots_limit.unwrap_or(600);

		Ok(Some(
			connections
				.into_iter()
				.map(|c| Some(UserConnection::from_db(c, slots)))
				.collect(),
		))
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::Edit)")]
	async fn editors(
		&self,
		ctx: &Context<'_>,
		editor_id: GqlObjectId,
		data: UserEditorUpdate,
	) -> Result<Option<Vec<Option<UserEditor>>>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let (_, authed_user_perms) = auth_session.user(global).await?;

		if !(auth_session.user_id() == self.id.id() || authed_user_perms.has(UserPermission::Admin)) {
			return Err(ApiError::FORBIDDEN);
		}

		if let Some(permissions) = data.permissions {
			if permissions == UserEditorModelPermission::none() {
				database::UserEditor::collection(global.db())
					.delete_one(
						doc! {
							"user_id": self.id.id(),
							"editor_id": editor_id.id(),
						},
						None,
					)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to delete editor");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			} else {
				database::UserEditor::collection(global.db())
					.update_one(
						doc! {
							"user_id": self.id.id(),
							"editor_id": editor_id.id(),
						},
						doc! {
							"$set": {
								"permissions": to_bson(&permissions.to_db()).map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?,
							},
						},
						None,
					)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to update editor");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}
		}

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

	#[graphql(guard = "PermissionGuard::one(UserPermission::Edit)")]
	async fn cosmetics(&self, update: UserCosmeticUpdate) -> Result<bool, ApiError> {
		// TODO: entitlements required
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(
		guard = "PermissionGuard::all([Permission::from(RolePermission::Assign), Permission::from(UserPermission::Edit)])"
	)]
	async fn roles(&self, action: ListItemAction) -> Result<RoleObjectId, ApiError> {
		// TODO: entitlements required
		Err(ApiError::NOT_IMPLEMENTED)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserConnectionUpdate {
	emote_set_id: Option<EmoteSetObjectId>,
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
	id: ObjectId<()>,
	kind: CosmeticKind,
	selected: bool,
}
