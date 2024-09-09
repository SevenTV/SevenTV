use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, UpdateOptions};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, RolePermission, UserPermission};
use shared::database::user::connection::UserConnection as DbUserConnection;
use shared::database::user::editor::{UserEditor as DbUserEditor, UserEditorId, UserEditorState};
use shared::database::user::{User, UserStyle};
use shared::database::MongoCollection;
use shared::event::{InternalEvent, InternalEventData, InternalEventUserData, InternalEventUserEditorData};
use shared::old_types::cosmetic::CosmeticKind;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::UserEditorModelPermission;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::session::Session;
use crate::http::v3::gql::guards::{PermissionGuard, RateLimitGuard, UserGuard};
use crate::http::v3::gql::queries::user::{UserConnection, UserEditor};
use crate::http::v3::gql::types::ListItemAction;
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
	#[graphql(
		guard = "RateLimitGuard::new(RateLimitResource::UserChangeConnections, 1).and(UserGuard(self.id.id()).or(PermissionGuard::one(UserPermission::ManageAny)))"
	)]
	async fn connections<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: String,
		data: UserConnectionUpdate,
	) -> Result<Option<Vec<Option<UserConnection>>>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let session = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let authed_user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

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

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::User {
						after: user.clone(),
						data: InternalEventUserData::RemoveConnection { connection },
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if let Some(emote_set) = emote_set {
				let old = if let Some(set_id) = old_user.user.style.active_emote_set_id {
					global
						.emote_set_by_id_loader
						.load(set_id)
						.await
						.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
				} else {
					None
				};

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::User {
						after: user.clone(),
						data: InternalEventUserData::ChangeActiveEmoteSet {
							old: old.map(Box::new),
							new: Some(Box::new(emote_set)),
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

	#[graphql(
		guard = "RateLimitGuard::new(RateLimitResource::UserChangeEditor, 1).and(UserGuard(self.id.id()).or(PermissionGuard::one(UserPermission::ManageAny)))"
	)]
	async fn editors(
		&self,
		ctx: &Context<'_>,
		editor_id: GqlObjectId,
		data: UserEditorUpdate,
	) -> Result<Option<Vec<Option<UserEditor>>>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let session = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let authed_user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

		let res = with_transaction(global, |mut tx| async move {
			// load all editors, we have to do this to know the old permissions Sadge
			let editors = global
				.user_editor_by_user_id_loader
				.load(authed_user.id)
				.await
				.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
				.unwrap_or_default();

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
						let editor_user = global
							.user_loader
							.load_fast(global, editor.id.editor_id)
							.await
							.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
							.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

						tx.register_event(InternalEvent {
							actor: Some(authed_user.clone()),
							session_id: session.user_session_id(),
							data: InternalEventData::UserEditor {
								after: editor,
								data: InternalEventUserEditorData::RemoveEditor {
									editor: Box::new(editor_user.user),
								},
							},
							timestamp: chrono::Utc::now(),
						})?;
					}
				} else {
					let old_permissions = editors
						.iter()
						.find(|e| e.id.editor_id == editor_id.id())
						.map(|e| e.permissions);

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
									permissions,
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
						let editor_user = global
							.user_loader
							.load_fast(global, editor.id.editor_id)
							.await
							.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
							.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

						tx.register_event(InternalEvent {
							actor: Some(authed_user.clone()),
							session_id: session.user_session_id(),
							data: InternalEventData::UserEditor {
								after: editor,
								data: InternalEventUserEditorData::EditPermissions {
									old: old_permissions.unwrap_or_default(),
									editor: Box::new(editor_user.user),
								},
							},
							timestamp: chrono::Utc::now(),
						})?;
					} else {
						// didn't exist
						if authed_user.has(UserPermission::InviteEditors) {
							return Err(TransactionError::custom(ApiError::FORBIDDEN));
						}

						let editor = DbUserEditor {
							id: editor_id,
							state: UserEditorState::Pending,
							notes: None,
							permissions,
							added_by_id: authed_user.id,
							added_at: chrono::Utc::now(),
							updated_at: chrono::Utc::now(),
							search_updated_at: None,
						};

						let editor_user = global
							.user_loader
							.load_fast(global, editor.id.editor_id)
							.await
							.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
							.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

						tx.insert_one::<DbUserEditor>(&editor, None).await?;

						tx.register_event(InternalEvent {
							actor: Some(authed_user.clone()),
							session_id: session.user_session_id(),
							data: InternalEventData::UserEditor {
								after: editor,
								data: InternalEventUserEditorData::AddEditor {
									editor: Box::new(editor_user.user),
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

	#[graphql(
		guard = "RateLimitGuard::new(RateLimitResource::UserChangeCosmetics, 1).and(UserGuard(self.id.id()).or(PermissionGuard::one(UserPermission::ManageAny)))"
	)]
	async fn cosmetics<'ctx>(&self, ctx: &Context<'ctx>, update: UserCosmeticUpdate) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let session = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let authed_user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

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

					let new = global
						.badge_by_id_loader
						.load(update.id.id())
						.await
						.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
						.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

					let old = if let Some(badge_id) = user.style.active_badge_id {
						global
							.badge_by_id_loader
							.load(badge_id)
							.await
							.map_err(|_| TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?
					} else {
						None
					};

					tx.register_event(InternalEvent {
						actor: Some(authed_user.clone()),
						session_id: session.user_session_id(),
						data: InternalEventData::User {
							after: user.user.clone(),
							data: InternalEventUserData::ChangeActiveBadge {
								old: old.map(Box::new),
								new: Some(Box::new(new)),
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
		})
		.await;

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
		let session = ctx.data::<Session>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let authed_user = session.user().ok_or(ApiError::UNAUTHORIZED)?;

		let role = global
			.role_by_id_loader
			.load(role_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		if authed_user.computed.permissions.is_superset_of(&role.permissions) {
			return Err(ApiError::FORBIDDEN);
		}

		let target_user = global
			.user_loader
			.load(global, self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		let res = with_transaction(global, |mut tx| async move {
			let roles = match action {
				ListItemAction::Add => {
					let edge_id = EntitlementEdgeId {
						from: EntitlementEdgeKind::User { user_id: self.id.id() },
						to: EntitlementEdgeKind::Role { role_id: role_id.id() },
						managed_by: None,
					};

					let res = tx
						.update_one(
							filter::filter! {
								EntitlementEdge {
									#[query(rename = "_id", serde)]
									id: &edge_id
								}
							},
							update::update! {
								#[query(set_on_insert)]
								EntitlementEdge {
									id: edge_id,
								}
							},
							Some(UpdateOptions::builder().upsert(true).build()),
						)
						.await?;

					if res.upserted_id.is_some() {
						tx.register_event(InternalEvent {
							actor: Some(authed_user.clone()),
							session_id: session.user_session_id(),
							data: InternalEventData::User {
								after: target_user.user.clone(),
								data: InternalEventUserData::AddEntitlement {
									target: EntitlementEdgeKind::Role { role_id: role_id.id() },
								},
							},
							timestamp: chrono::Utc::now(),
						})?;
					}

					let no_role = !target_user.computed.roles.contains(&role_id.id());

					target_user
						.computed
						.entitlements
						.roles
						.iter()
						.copied()
						// If the user didnt have the role before, we add it
						.chain(no_role.then_some(role_id.id()))
						.map(Into::into)
						.collect()
				}
				ListItemAction::Remove => {
					if tx
						.delete_one(
							filter::filter! {
								EntitlementEdge {
									#[query(serde)]
									id: EntitlementEdgeId {
										from: EntitlementEdgeKind::User { user_id: self.id.id() },
										to: EntitlementEdgeKind::Role { role_id: role_id.id() },
										managed_by: None,
									}
								}
							},
							None,
						)
						.await?
						.deleted_count == 1
					{
						tx.register_event(InternalEvent {
							actor: Some(authed_user.clone()),
							session_id: session.user_session_id(),
							data: InternalEventData::User {
								after: target_user.user.clone(),
								data: InternalEventUserData::RemoveEntitlement {
									target: EntitlementEdgeKind::Role { role_id: role_id.id() },
								},
							},
							timestamp: chrono::Utc::now(),
						})?;
					};

					// They might have the role via some other entitlement.
					let role_via_edge = target_user.computed.raw_entitlements.iter().flat_map(|e| e.iter()).any(|e| {
						e.id.to == EntitlementEdgeKind::Role { role_id: role_id.id() }
							&& (e.id.from != EntitlementEdgeKind::User { user_id: self.id.id() }
								|| e.id.managed_by.is_some())
					});

					target_user
						.computed
						.entitlements
						.roles
						.iter()
						.copied()
						.filter(|id| role_via_edge || *id != role_id.id())
						.map(Into::into)
						.collect()
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
