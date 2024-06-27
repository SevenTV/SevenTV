use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::bson::{doc, to_bson};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, UpdateOptions};
use shared::database::role::permissions::{PermissionsExt, RolePermission, UserPermission};
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

		if let Some(permissions) = data.permissions {
			if permissions == UserEditorModelPermission::none() {
				// Remove editor
				shared::database::user::editor::UserEditor::collection(global.db())
					.delete_one(
						doc! {
							"user_id": self.id.0,
							"editor_id": editor_id.0,
						},
						None,
					)
					.await
					.map_err(|err| {
						tracing::error!(error = %err, "failed to delete editor");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			} else {
				// Add or update editor
				shared::database::user::editor::UserEditor::collection(global.db())
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
						UpdateOptions::builder()
							.upsert(user.has(UserPermission::InviteEditors))
							.build(),
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

	async fn cosmetics(&self, update: UserCosmeticUpdate) -> Result<bool, ApiError> {
		// TODO: entitlements required
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Assign)")]
	async fn roles(&self, action: ListItemAction) -> Result<GqlObjectId, ApiError> {
		// TODO: entitlements required
		Err(ApiError::NOT_IMPLEMENTED)
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
