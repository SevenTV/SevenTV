use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::event::{EventEntitlementEdgeData, EventUserData, EventUserEditorData};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{AdminPermission, PermissionsExt, RolePermission, UserPermission};
use shared::database::user::connection::UserConnection as DbUserConnection;
use shared::database::user::editor::{UserEditor as DbUserEditor, UserEditorId, UserEditorState};
use shared::database::user::{User, UserStyle};
use shared::database::MongoCollection;
use shared::event::{EventPayload, EventPayloadData};
use shared::old_types::cosmetic::CosmeticKind;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::user::{UserConnection, UserEditor};
use crate::http::v3::gql::types::ListItemAction;
use crate::http::v3::types::UserEditorModelPermission;
use crate::transactions::{with_transaction, TransactionError};

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

		let authed_user = auth_session.user(global).await?;

		if !(auth_session.user_id() == self.id.id() || authed_user.has(UserPermission::ManageAny)) {
			return Err(ApiError::FORBIDDEN);
		}

		let res = with_transaction(global, |mut tx| async move {
			let old_user = global
				.user_loader
				.load(global, self.id.id())
				.await
				.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
				.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

			let emote_set = if let Some(emote_set_id) = data.emote_set_id {
				// check if set exists
				let emote_set = global
					.emote_set_by_id_loader
					.load(emote_set_id.id())
					.await
					.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
					.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

				Some(emote_set)
			} else {
				None
			};

			let update_pull = data.unlink.is_some_and(|u| u).then_some(update::update! {
				#[query(pull)]
				User {
					connections: DbUserConnection {
						platform_id: id.clone(),
					}
				}
			});

			let Some(user) = tx
				.find_one_and_update(
					filter::filter! {
						User {
							#[query(rename = "_id")]
							id: self.id.id(),
						}
					},
					update::update! {
						#[query(set)]
						User {
							#[query(flatten)]
							style: UserStyle {
								#[query(optional)]
								active_emote_set_id: data.emote_set_id.map(|id| id.id()),
							},
							updated_at: chrono::Utc::now(),
						},
						#[query(pull)]
						update_pull,
					},
					FindOneAndUpdateOptions::builder()
						.return_document(ReturnDocument::After)
						.build(),
				)
				.await?
			else {
				return Ok(None);
			};

			if let Some(true) = data.unlink {
				let connection = old_user
					.user
					.connections
					.into_iter()
					.find(|c| c.platform_id == id)
					.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

				tx.register_event(EventPayload {
					actor_id: Some(authed_user.id),
					data: EventPayloadData::User {
						after: user.clone(),
						data: EventUserData::RemoveConnection { connection },
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if let Some(emote_set) = emote_set {
				tx.register_event(EventPayload {
					actor_id: Some(authed_user.id),
					data: EventPayloadData::User {
						after: user.clone(),
						data: EventUserData::ChangeActiveEmoteSet {
							old: old_user.user.style.active_emote_set_id,
							new: Some(emote_set.id),
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			Ok(Some(user))
		})
		.await;

		match res {
			Ok(Some(user)) => {
				let full_user = global
					.user_loader
					.load_fast_user(global, user)
					.await
					.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

				Ok(Some(
					full_user
						.connections
						.iter()
						.cloned()
						.map(|c| {
							Some(UserConnection::from_db(
								full_user.computed.permissions.emote_set_capacity.unwrap_or_default(),
								c,
								&full_user.style,
							))
						})
						.collect(),
				))
			}
			Ok(None) => Ok(None),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
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

		let res = with_transaction(global, |mut tx| async move {
			if let Some(permissions) = data.permissions {
				if permissions == UserEditorModelPermission::none() {
					let editor_id = UserEditorId {
						user_id: self.id.id(),
						editor_id: editor_id.id(),
					};

					// Remove editor
					let res = tx
						.find_one_and_delete(
							filter::filter! {
								DbUserEditor {
									#[query(serde, rename = "_id")]
									id: editor_id,
								}
							},
							None,
						)
						.await?;

					if let Some(editor) = res {
						tx.register_event(EventPayload {
							actor_id: Some(auth_session.user_id()),
							data: EventPayloadData::UserEditor {
								after: editor,
								data: EventUserEditorData::RemoveEditor {
									editor_id: editor_id.editor_id,
								},
							},
							timestamp: chrono::Utc::now(),
						})?;
					}
				} else {
					// Add or update editor
					let editor_id = UserEditorId {
						user_id: self.id.id(),
						editor_id: editor_id.id(),
					};

					let permissions = permissions.to_db();

					let editor = tx
						.find_one_and_update(
							filter::filter! {
								DbUserEditor {
									#[query(serde, rename = "_id")]
									id: editor_id,
								}
							},
							update::update! {
								#[query(set)]
								DbUserEditor {
									#[query(serde)]
									permissions: permissions.clone(),
									updated_at: chrono::Utc::now(),
								}
							},
							FindOneAndUpdateOptions::builder()
								.return_document(ReturnDocument::After)
								.build(),
						)
						.await?;

					if let Some(editor) = editor {
						// updated
						tx.register_event(EventPayload {
							actor_id: Some(auth_session.user_id()),
							data: EventPayloadData::UserEditor {
								after: editor,
								data: EventUserEditorData::EditPermissions {
									old: todo!("query old permissions"),
									new: permissions,
								},
							},
							timestamp: chrono::Utc::now(),
						})?;
					} else {
						// didn't exist
						if user.has(UserPermission::InviteEditors) {
							return Err(TransactionError::custom(ApiError::FORBIDDEN));
						}

						let editor = DbUserEditor {
							id: editor_id,
							state: UserEditorState::Pending,
							notes: None,
							permissions: permissions.clone(),
							added_by_id: auth_session.user_id(),
							added_at: chrono::Utc::now(),
							updated_at: chrono::Utc::now(),
							search_updated_at: None,
						};

						tx.insert_one::<DbUserEditor>(&editor, None).await?;

						tx.register_event(EventPayload {
							actor_id: Some(auth_session.user_id()),
							data: EventPayloadData::UserEditor {
								after: editor,
								data: EventUserEditorData::AddEditor {
									editor_id: editor_id.editor_id,
								},
							},
							timestamp: chrono::Utc::now(),
						})?;
					}
				}
			}

			Ok(())
		})
		.await;

		match res {
			Ok(_) => {
				let editors = global
					.user_editor_by_user_id_loader
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
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
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
			.user_loader
			.load(global, self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		let res = with_transaction(global, |mut tx| async move {
			match update.kind {
				CosmeticKind::Paint => {
					// check if user has paint
					if !user.computed.entitlements.paints.contains(&update.id.id()) {
						return Err(TransactionError::custom(ApiError::FORBIDDEN));
					}

					let res = User::collection(&global.db)
						.update_one(
							filter::filter! {
								User {
									#[query(rename = "_id")]
									id: self.id.id(),
								}
							},
							update::update! {
								#[query(set)]
								User {
									#[query(flatten)]
									style: UserStyle {
										active_paint_id: update.id.id(),
									}
								}
							},
						)
						.await?;

					Ok(res.modified_count == 1)
				}
				CosmeticKind::Badge => {
					// check if user has paint
					if !user.computed.entitlements.badges.contains(&update.id.id()) {
						return Err(TransactionError::custom(ApiError::FORBIDDEN));
					}

					tx.register_event(EventPayload {
						actor_id: Some(authed_user.id),
						data: EventPayloadData::User {
							after: user.user.clone(),
							data: EventUserData::ChangeActiveBadge {
								old: user.style.active_badge_id,
								new: Some(update.id.id()),
							},
						},
						timestamp: chrono::Utc::now(),
					})?;

					let res = User::collection(&global.db)
						.update_one(
							filter::filter! {
								User {
									#[query(rename = "_id")]
									id: self.id.id(),
								}
							},
							update::update! {
								#[query(set)]
								User {
									#[query(flatten)]
									style: UserStyle {
										active_badge_id: update.id.id(),
									}
								},
							},
						)
						.await?;

					Ok(res.modified_count == 1)
				}
				CosmeticKind::Avatar => Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED)),
			}
		}).await;

		match res {
			Ok(b) => Ok(b),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
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
			.role_by_id_loader
			.load(role_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		if role.permissions > user.computed.permissions && !user.has(AdminPermission::SuperAdmin) {
			return Err(ApiError::FORBIDDEN);
		}

		let res = with_transaction(global, |mut tx| async move {
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

					tx.insert_one::<EntitlementEdge>(&edge, None).await?;

					tx.register_event(EventPayload {
						actor_id: Some(auth_session.user_id()),
						data: EventPayloadData::EntitlementEdge {
							after: edge,
							data: EventEntitlementEdgeData::Create,
						},
						timestamp: chrono::Utc::now(),
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
					if let Some(edge) = tx
						.find_one_and_delete(
							filter::filter! {
								EntitlementEdge {
									#[query(serde)]
									id: EntitlementEdgeId {
										from: EntitlementEdgeKind::User { user_id: self.id.id() }.into(),
										to: EntitlementEdgeKind::Role { role_id: role_id.id() }.into(),
										managed_by: None,
									}
								}
							},
							None,
						)
						.await?
					{
						tx.register_event(EventPayload {
							actor_id: Some(auth_session.user_id()),
							data: EventPayloadData::EntitlementEdge {
								after: edge,
								data: EventEntitlementEdgeData::Delete,
							},
							timestamp: chrono::Utc::now(),
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
				ListItemAction::Update => return Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED)),
			};

			Ok(roles)
		})
		.await;

		match res {
			Ok(roles) => Ok(roles),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
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
