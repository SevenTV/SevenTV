use std::collections::HashSet;
use std::convert::Infallible;
use std::ops::Deref;
use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, UpdateOptions};
use shared::database::badge::BadgeId;
use shared::database::emote::Emote;
use shared::database::emote_set::{EmoteSet, EmoteSetKind};
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind, EntitlementEdgeManagedBy};
use shared::database::paint::PaintId;
use shared::database::product::invoice::{Invoice, InvoiceStatus};
use shared::database::product::subscription::{
	SubscriptionId, SubscriptionPeriod, SubscriptionPeriodCreatedBy, SubscriptionPeriodId,
};
use shared::database::product::{InvoiceId, SubscriptionProductKind};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, RolePermission, UserPermission};
use shared::database::user::ban::UserBan;
use shared::database::user::connection::UserConnection as DbUserConnection;
use shared::database::user::editor::{EditorUserPermission, UserEditor as DbUserEditor, UserEditorId, UserEditorState};
use shared::database::user::session::UserSession;
use shared::database::user::{User, UserId, UserStyle};
use shared::database::{Id, MongoCollection};
use shared::event::{InternalEvent, InternalEventData, InternalEventUserData, InternalEventUserEditorData};
use shared::old_types::cosmetic::CosmeticKind;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::UserEditorModelPermission;
use tracing::Instrument;

use crate::global::Global;
use crate::http::egvault::metadata::{InvoiceMetadata, StripeMetadata};
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::middleware::session::Session;
use crate::http::v3::gql::queries::user::{UserConnection, UserEditor};
use crate::http::v3::gql::types::ListItemAction;
use crate::sub_refresh_job;
use crate::transactions::{transaction, transaction_with_mutex, GeneralMutexKey, TransactionError};

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

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(rename_items = "snake_case")]
pub enum SubscriptionPeriodKind {
	Yearly,
	Monthly,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct SubscriptionPeriodCreate {
	product_id: GqlObjectId,
	kind: SubscriptionPeriodKind,
	start: chrono::DateTime<chrono::Utc>,
	end: chrono::DateTime<chrono::Utc>,
	reason: String,
	invoice: Option<SubscriptionPeriodCreateInvoiceData>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct SubscriptionPeriodCreateInvoiceData {
	price: f64,
	currency: Option<String>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateSubscriptionPeriodResponse {
	success: bool,
	invoice_id: Option<String>,
	payment_declined: bool,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl UserOps {
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::UserChangeConnections, 1)")]
	#[tracing::instrument(skip_all, name = "UserOps::connections")]
	async fn connections<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: String,
		data: UserConnectionUpdate,
	) -> Result<Option<Vec<Option<UserConnection>>>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.id.id() && !authed_user.has(UserPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id: self.id.id(),
				})
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"you do not have permission to modify connections",
					)
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(EditorUserPermission::ManageProfile) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to modify connections, you need the ManageProfile permission",
				));
			}
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::User(self.id.id()).into()),
			|mut tx| async move {
				let old_user = global
					.user_loader
					.load(global, self.id.id())
					.await
					.map_err(|_| {
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to load user",
						))
					})?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))
					})?;

				let emote_set = if let Some(emote_set_id) = data.emote_set_id {
					// check if set exists
					let emote_set = global
						.emote_set_by_id_loader
						.load(emote_set_id.id())
						.await
						.map_err(|_| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load emote set",
							))
						})?
						.ok_or_else(|| {
							TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))
						})?;

					if emote_set.kind != EmoteSetKind::Normal {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::BadRequest,
							"emote set is not a normal set",
						)));
					}

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
								search_updated_at: &None,
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
					if user.connections.is_empty() {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::BadRequest,
							"cannot remove last connection",
						)));
					}

					let connection = old_user
						.user
						.connections
						.into_iter()
						.find(|c| c.platform_id == id)
						.ok_or_else(|| {
							TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "connection not found"))
						})?;

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
						global.emote_set_by_id_loader.load(set_id).await.map_err(|_| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load emote set",
							))
						})?
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
			},
		)
		.await;

		match res {
			Ok(Some(user)) => {
				let full_user = global
					.user_loader
					.load_fast_user(global, user)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

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
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::UserChangeEditor, 1)")]
	#[tracing::instrument(skip_all, name = "UserOps::editors")]
	async fn editors(
		&self,
		ctx: &Context<'_>,
		editor_id: GqlObjectId,
		data: UserEditorUpdate,
	) -> Result<Option<Vec<Option<UserEditor>>>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if !authed_user.has(UserPermission::InviteEditors) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you are not allowed to invite editors",
			));
		}

		let permissions = data.permissions.unwrap_or(UserEditorModelPermission::none());

		// They should be able to remove themselves from the editor list
		if authed_user.id != self.id.id()
			&& !authed_user.has(UserPermission::ManageAny)
			&& !(editor_id.id() == authed_user.id() && permissions.is_none())
		{
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id: self.id.id(),
				})
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"you do not have permission to modify editors",
					)
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(EditorUserPermission::ManageEditors) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to modify editors, you need the ManageEditors permission",
				));
			}
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::User(self.id.id()).into()),
			|mut tx| async move {
				let editor_id = UserEditorId {
					user_id: self.id.id(),
					editor_id: editor_id.id(),
				};

				if permissions.is_none() {
					// Remove editor
					let res = tx
						.find_one_and_delete(
							filter::filter! {
								DbUserEditor {
									#[query(rename = "_id", serde)]
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
							.map_err(|_| {
								TransactionError::Custom(ApiError::internal_server_error(
									ApiErrorCode::LoadError,
									"failed to load user",
								))
							})?
							.ok_or_else(|| {
								TransactionError::Custom(ApiError::internal_server_error(
									ApiErrorCode::LoadError,
									"failed to load user",
								))
							})?;

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
					let old_permissions = tx
						.find_one(
							filter::filter! {
								DbUserEditor {
									#[query(rename = "_id", serde)]
									id: editor_id,
								}
							},
							None,
						)
						.await?
						.as_ref()
						.map(|e| e.permissions);

					// Add or update editor
					let permissions = permissions.to_db();

					if old_permissions == Some(permissions) {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::BadRequest,
							"permissions are the same",
						)));
					}

					let now = chrono::Utc::now();

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
									search_updated_at: &None,
								},
								#[query(set_on_insert)]
								DbUserEditor {
									#[query(serde)]
									state: UserEditorState::Pending,
									notes: None,
									added_at: now,
									added_by_id: authed_user.id,
								}
							},
							FindOneAndUpdateOptions::builder()
								.upsert(true)
								.return_document(ReturnDocument::After)
								.build(),
						)
						.await?
						.ok_or_else(|| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load editor",
							))
						})?;

					// updated
					let editor_user = global
						.user_loader
						.load_fast(global, editor.id.editor_id)
						.await
						.map_err(|_| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load user",
							))
						})?
						.ok_or_else(|| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load user",
							))
						})?;

					if old_permissions.is_none() {
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
					} else {
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
					}
				}

				Ok(())
			},
		)
		.await;

		match res {
			Ok(_) => {
				let editors = global
					.user_editor_by_user_id_loader
					.load(self.id.id())
					.await
					.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
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
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::UserChangeCosmetics, 1)")]
	#[tracing::instrument(skip_all, name = "UserOps::cosmetics")]
	async fn cosmetics<'ctx>(&self, ctx: &Context<'ctx>, update: UserCosmeticUpdate) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if !update.selected {
			return Ok(true);
		}

		if authed_user.id != self.id.id() && !authed_user.has(UserPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id: self.id.id(),
				})
				.await
				.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"you do not have permission to change this user's cosmetics",
					)
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(EditorUserPermission::ManageProfile) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to modify this user's cosmetics, you need the ManageProfile permission",
				));
			}
		}

		let user = global
			.user_loader
			.load(global, self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		let res =
			transaction_with_mutex(
				global,
				Some(GeneralMutexKey::User(self.id.id()).into()),
				|mut tx| async move {
					match update.kind {
				CosmeticKind::Paint => {
					let id: Option<PaintId> = if update.id.0.is_nil() { None } else { Some(update.id.id()) };

					// check if user has paint
					if id.is_some_and(|id| !user.computed.entitlements.paints.contains(&id)) {
						return Err(TransactionError::Custom(ApiError::forbidden(
							ApiErrorCode::LoadError,
							"you do not have permission to use this paint",
						)));
					}

					if user.style.active_paint_id == id {
						return Ok(true);
					}

					let new = if let Some(id) = id {
						Some(
							global
								.paint_by_id_loader
								.load(id)
								.await
								.map_err(|_| {
									TransactionError::Custom(ApiError::internal_server_error(
										ApiErrorCode::LoadError,
										"failed to load paint",
									))
								})?
								.ok_or_else(|| {
									TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "paint not found"))
								})?,
						)
					} else {
						None
					};

					let old = if let Some(paint_id) = user.style.active_paint_id {
						global.paint_by_id_loader.load(paint_id).await.map_err(|_| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load badge",
							))
						})?
					} else {
						None
					};

					tx.register_event(InternalEvent {
						actor: Some(authed_user.clone()),
						session_id: session.user_session_id(),
						data: InternalEventData::User {
							after: user.user.clone(),
							data: InternalEventUserData::ChangeActivePaint {
								old: old.map(Box::new),
								new: new.map(Box::new),
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
										active_paint_id: id,
									},
									updated_at: chrono::Utc::now(),
									search_updated_at: &None,
								}
							},
						)
						.await?;

					Ok(res.modified_count == 1)
				}
				CosmeticKind::Badge => {
					let id: Option<BadgeId> = if update.id.0.is_nil() { None } else { Some(update.id.id()) };

					// check if user has paint
					if id.is_some_and(|id| !user.computed.entitlements.badges.contains(&id)) {
						return Err(TransactionError::Custom(ApiError::forbidden(
							ApiErrorCode::LoadError,
							"you do not have permission to use this badge",
						)));
					}

					if user.style.active_badge_id == id {
						return Ok(true);
					}

					let new = if let Some(id) = id {
						Some(
							global
								.badge_by_id_loader
								.load(id)
								.await
								.map_err(|_| {
									TransactionError::Custom(ApiError::internal_server_error(
										ApiErrorCode::LoadError,
										"failed to load badge",
									))
								})?
								.ok_or_else(|| {
									TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "badge not found"))
								})?,
						)
					} else {
						None
					};

					let old = if let Some(badge_id) = user.style.active_badge_id {
						global.badge_by_id_loader.load(badge_id).await.map_err(|_| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load badge",
							))
						})?
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
								new: new.map(Box::new),
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
										active_badge_id: id,
									},
									updated_at: chrono::Utc::now(),
									search_updated_at: &None,
								},
							},
						)
						.await?;

					Ok(res.modified_count == 1)
				}
				CosmeticKind::Avatar => Err(TransactionError::Custom(ApiError::not_implemented(
					ApiErrorCode::BadRequest,
					"avatar cosmetics mutations are not supported via this endpoint, use the upload endpoint instead",
				))),
			}
				},
			)
			.await;

		match res {
			Ok(b) => Ok(b),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(guard = "PermissionGuard::one(RolePermission::Assign)")]
	#[tracing::instrument(skip_all, name = "UserOps::roles")]
	async fn roles<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		role_id: GqlObjectId,
		action: ListItemAction,
	) -> Result<Vec<GqlObjectId>, ApiError> {
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

		let target_user = global
			.user_loader
			.load(global, self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::User(self.id.id()).into()),
			|mut tx| async move {
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
										#[query(serde, rename = "_id")]
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
										#[query(rename = "_id", serde)]
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
						}

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
					ListItemAction::Update => {
						return Err(TransactionError::Custom(ApiError::not_implemented(
							ApiErrorCode::BadRequest,
							"update role is not implemented",
						)));
					}
				};

				Ok(roles)
			},
		)
		.await;

		match res {
			Ok(roles) => Ok(roles),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageAny)")]
	#[tracing::instrument(skip_all, name = "UserOps::merge")]
	async fn merge<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		let authed_user = session.user()?;

		let src_user_id: UserId = self.id.id();
		let target_user_id: UserId = id.id();

		if src_user_id == target_user_id {
			return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "cannot merge user with self"));
		}

		// SO currently we dont have a way to undo a merge, so we just log the merge
		// and if someone fucks up they will have to DM me to undo it - Troy
		// This is also not perfect as we dont merge all the objects that reference
		// users We merge:
		// - Connections
		// - Emote Sets
		// - Subscription Periods
		// - UserBans
		// - Emotes
		// - Editors
		let subscription_products = global
			.subscription_products_loader
			.load(())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load subscription products"))?
			.unwrap_or_default();

		let subscription_ids =
			transaction_with_mutex(global, Some(GeneralMutexKey::User(src_user_id).into()), |mut tx| async move {
				let src_user = tx
					.find_one(
						filter::filter! {
							User {
								#[query(rename = "_id")]
								id: src_user_id,
							}
						},
						None,
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))
					})?;

				let target_user = tx
					.find_one(
						filter::filter! {
							User {
								#[query(rename = "_id")]
								id: target_user_id,
							}
						},
						None,
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))
					})?;

				if target_user.merged_into_id.is_some() {
					return Err(TransactionError::Custom(ApiError::bad_request(ApiErrorCode::BadRequest, "target user is already merged")));
				}

				let mut update = update::update! {
					#[query(push)]
					User {
						#[query(each, serde)]
						connections: &src_user.connections,
					},
					#[query(set)]
					User {
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					}
				};

				if let (Some(stripe_customer_id), None) = (src_user.stripe_customer_id, target_user.stripe_customer_id) {
					update = update.extend_one(update::update! {
						#[query(set)]
						User {
							stripe_customer_id,
						}
					});
				}

				if src_user.has_bans {
					update = update.extend_one(update::update! {
						#[query(set)]
						User {
							has_bans: true,
						}
					});
				}

				if target_user.email.is_none() {
					update = update.extend_one(update::update! {
						#[query(set)]
						User {
							email: src_user.email,
						}
					});
				}

				tx.update_one(
					filter::filter! {
						User {
							#[query(rename = "_id")]
							id: src_user_id,
						}
					},
					update::update! {
						#[query(set)]
						User {
							merged_into_id: Some(target_user_id),
							#[query(serde)]
							connections: Vec::new(),
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await?;

				let after = tx
					.find_one_and_update(
						filter::filter! {
							User {
								#[query(rename = "_id")]
								id: target_user_id,
							}
						},
						update,
						Some(
							FindOneAndUpdateOptions::builder()
								.return_document(ReturnDocument::After)
								.build(),
						),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))
					})?;

				tx
					.update(
						filter::filter! {
							EmoteSet {
								owner_id: src_user_id,
								#[query(serde)]
								kind: EmoteSetKind::Normal,
							}
						},
						update::update! {
							#[query(set)]
							EmoteSet {
								owner_id: target_user_id,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						None,
					)
					.await?;

				let periods = tx
					.update(
						filter::filter! {
							SubscriptionPeriod {
								#[query(flatten)]
								subscription_id: SubscriptionId {
									user_id: src_user_id,
								},
							}
						},
						update::update! {
							#[query(set)]
							SubscriptionPeriod {
								#[query(flatten)]
								subscription_id: SubscriptionId {
									user_id: target_user_id,
								},
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						None,
					)
					.await?;

				tx.update(
					filter::filter! {
						UserBan {
							user_id: src_user_id,
						}
					},
					update::update! {
						#[query(set)]
						UserBan {
							user_id: target_user_id,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await?;

				tx.update(
					filter::filter! {
						Emote {
							owner_id: src_user_id,
						}
					},
					update::update! {
						#[query(set)]
						Emote {
							owner_id: target_user_id,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await?;

				let editors = tx
					.find(
						filter::filter! {
							DbUserEditor {
								#[query(flatten, rename = "_id")]
								id: UserEditorId {
									user_id: src_user_id,
								},
							}
						},
						None,
					)
					.await?;

				let target_editors = tx
					.find(
						filter::filter! {
							DbUserEditor {
								#[query(flatten, rename = "_id")]
								id: UserEditorId {
									user_id: target_user_id,
								},
							}
						},
						None,
					)
					.await?
					.into_iter()
					.map(|e| e.id.editor_id)
					.collect::<HashSet<_>>();

				let new_editors = editors
					.into_iter()
					.filter(|e| !target_editors.contains(&e.id.editor_id) && e.id.user_id != target_user_id)
					.map(|mut e| {
						e.id.user_id = target_user_id;
						e
					})
					.collect::<Vec<_>>();

				if !new_editors.is_empty() {
					tx.insert_many(
						new_editors,
						None,
					)
					.await?;
				}

				tx.delete(
					filter::filter! {
						DbUserEditor {
							#[query(flatten, rename = "_id")]
							id: UserEditorId {
								user_id: src_user_id,
							},
						}
					},
					None,
				)
				.await?;

				let editors_of_src = tx.find(
					filter::filter! {
						DbUserEditor {
							#[query(flatten, rename = "_id")]
							id: UserEditorId {
								editor_id: src_user_id,
							},
						}
					},
					None,
				)
				.await?;

				let editors_of_target = tx.find(
					filter::filter! {
						DbUserEditor {
							#[query(flatten, rename = "_id")]
							id: UserEditorId {
								editor_id: target_user_id,
							},
						}
					},
					None,
				)
				.await?
				.into_iter()
				.map(|e| e.id.user_id)
				.collect::<HashSet<_>>();

				tx.delete(
					filter::filter! {
						DbUserEditor {
							#[query(flatten, rename = "_id")]
							id: UserEditorId {
								editor_id: src_user_id,
							},
						}
					},
					None,
				)
				.await?;

				let items = editors_of_src.into_iter().filter_map(|mut e| {
					if editors_of_target.contains(&e.id.user_id) {
						return None;
					}

					e.id.editor_id = target_user_id;

					Some(e)
				})
				.collect::<Vec<_>>();

				if !items.is_empty() {
					tx.insert_many(
						items,
						None,
					)
					.await?;
				}

				tx.delete(
					filter::filter! {
						UserSession {
							user_id: src_user_id,
						}
					},
					None,
				)
				.await?;

				let source_edges_filter = filter::filter! {
					EntitlementEdge {
						#[query(flatten, rename = "_id")]
						id: EntitlementEdgeId {
							#[query(serde, selector = "in")]
							from: std::iter::once(EntitlementEdgeKind::User { user_id: src_user_id }).chain(subscription_products.iter().map(|p| EntitlementEdgeKind::Subscription { subscription_id: SubscriptionId { product_id: p.id, user_id: src_user_id } })).collect::<Vec<_>>(),
						}
					}
				};

				let mut source_edges = tx.find(
					source_edges_filter.clone(),
					None,
				)
				.await?
				.into_iter()
				.filter_map(|mut e| {
					if matches!(e.id.managed_by, Some(EntitlementEdgeManagedBy::Subscription { .. })) {
						return None;
					}

					match &mut e.id.from {
						EntitlementEdgeKind::User { user_id } => { *user_id = target_user_id; },
						EntitlementEdgeKind::Subscription { subscription_id } => { subscription_id.user_id = target_user_id; },
						_ => {},
					}

					if let EntitlementEdgeKind::Subscription { subscription_id } = &mut e.id.to { subscription_id.user_id = target_user_id; }

					Some(e)
				})
				.collect::<HashSet<_>>();

				let target_edges = tx.find(
					filter::filter! {
						EntitlementEdge {
							#[query(flatten, rename = "_id")]
							id: EntitlementEdgeId {
								#[query(serde, selector = "in")]
								from: std::iter::once(EntitlementEdgeKind::User { user_id: target_user_id }).chain(subscription_products.iter().map(|p| EntitlementEdgeKind::Subscription { subscription_id: SubscriptionId { product_id: p.id, user_id: target_user_id } })).collect::<Vec<_>>(),
							},
						}
					},
					None,
				)
				.await?;

				for e in target_edges {
					source_edges.remove(&e);
				}

				if !source_edges.is_empty() {
					tx.insert_many(source_edges, None).await?;
				}

				tx.delete(
					source_edges_filter,
					None,
				)
				.await?;

				tx.register_event(InternalEvent {
					timestamp: chrono::Utc::now(),
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::User {
						after,
						data: InternalEventUserData::Merge {
							connections: src_user.connections,
							source_id: src_user_id,
						},
					},
				})?;

				if periods.modified_count > 0 {
					Ok(tx
						.find(
							filter::filter! {
								SubscriptionPeriod {
									#[query(flatten)]
									subscription_id: SubscriptionId {
										user_id: target_user_id,
									},
								}
							},
							None,
						)
						.await?
						.into_iter()
						.map(|s| s.subscription_id)
						.collect::<HashSet<_>>())
				} else {
					Ok(HashSet::new())
				}
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "transaction failed");
				ApiError::internal_server_error(ApiErrorCode::TransactionError, "transaction failed")
			})?;

		for subscription_id in subscription_ids {
			sub_refresh_job::refresh(global, subscription_id).await?;
		}

		Ok(true)
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageAny)")]
	async fn delete(&self, ctx: &Context<'_>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		transaction(global, |mut tx| async move {
			let user = tx
				.find_one_and_delete(
					filter::filter! {
						User {
							#[query(rename = "_id")]
							id: self.id.id(),
						}
					},
					None,
				)
				.await?
				.ok_or_else(|| TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found")))?;

			tx.register_event(InternalEvent {
				timestamp: chrono::Utc::now(),
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::User {
					after: user,
					data: InternalEventUserData::Delete,
				},
			})?;

			Ok::<_, TransactionError<ApiError>>(())
		})
		.await
		.map_err(|e| {
			tracing::error!(error = %e, "transaction failed");
			ApiError::internal_server_error(ApiErrorCode::TransactionError, "transaction failed")
		})?;

		Ok(true)
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageAny)")]
	async fn create_subscription_period(
		&self,
		ctx: &Context<'_>,
		create: SubscriptionPeriodCreate,
	) -> Result<CreateSubscriptionPeriodResponse, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let product = global
			.subscription_product_by_id_loader
			.load(create.product_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load product"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "product not found"))?;

		let kind = match create.kind {
			SubscriptionPeriodKind::Yearly => SubscriptionProductKind::Yearly,
			SubscriptionPeriodKind::Monthly => SubscriptionProductKind::Monthly,
		};

		let price = product
			.variants
			.iter()
			.find(|v| v.kind == kind && v.active)
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "product variant not found"))?;

		let user = global
			.user_by_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		if let Some(invoice_create) = create.invoice {
			#[derive(Debug, Clone)]
			pub enum StripeRequest {
				CreateInvoice,
				CreateInvoiceItem,
				FinalizeInvoice,
				PayInvoice,
			}

			impl std::fmt::Display for StripeRequest {
				fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
					match self {
						Self::CreateInvoice => write!(f, "create_invoice"),
						Self::CreateInvoiceItem => write!(f, "add_invoice_item"),
						Self::FinalizeInvoice => write!(f, "finalize_invoice"),
						Self::PayInvoice => write!(f, "pay_invoice"),
					}
				}
			}

			let Some(customer_id) = user.stripe_customer_id else {
				return Err(ApiError::bad_request(
					ApiErrorCode::BadRequest,
					"user does not have a stripe customer id, ask them to setup a payment method first.",
				));
			};

			let customer = stripe::Customer::retrieve(global.stripe_client.client().await.deref(), &customer_id, &[])
				.instrument(tracing::info_span!("retrieve_customer"))
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to retrieve customer");
					ApiError::internal_server_error(ApiErrorCode::StripeError, "failed to retrieve customer")
				})?;

			let payment_method_id = customer
				.invoice_settings
				.and_then(|i| i.default_payment_method)
				.map(|method| method.id().as_str().to_owned())
				.or_else(|| customer.default_source.map(|s| s.id().as_str().to_owned()))
				.ok_or_else(|| {
					ApiError::bad_request(ApiErrorCode::BadRequest, "customer does not have a payment method set")
				})?;

			let currency = match invoice_create.currency {
				Some(c) => c
					.to_lowercase()
					.parse::<stripe::Currency>()
					.map_err(|_| ApiError::bad_request(ApiErrorCode::BadRequest, "invalid currency code"))?,
				None => product.default_currency,
			};

			let stripe_client = global.stripe_client.clone().safe(Id::<()>::new()).await;

			let status = transaction(global, |mut tx| async move {
				let invoice = stripe::Invoice::create(
					stripe_client.client(StripeRequest::CreateInvoice).await.deref(),
					stripe::CreateInvoice {
						customer: Some(customer_id.clone().into()),
						default_payment_method: Some(payment_method_id.as_str()),
						auto_advance: Some(true),
						description: Some(&create.reason),
						currency: Some(currency),
						metadata: Some(
							InvoiceMetadata::BoughtPeriod {
								user_id: self.id.id(),
								start: create.start,
								end: create.end,
								product_id: price.id.clone(),
								subscription_product_id: product.id,
							}
							.to_stripe(),
						),
						..Default::default()
					},
				)
				.instrument(tracing::info_span!("create_invoice"))
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to create invoice");
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to create invoice",
					))
				})?;

				let mut params = stripe::CreateInvoiceItem::new(customer_id.clone().into());
				params.invoice = Some(invoice.id.clone());
				params.price_data = Some(stripe::InvoiceItemPriceData {
					currency,
					product: product.provider_id.to_string(),
					unit_amount: Some((invoice_create.price * 100.0).round() as i64),
					..Default::default()
				});

				stripe::InvoiceItem::create(stripe_client.client(StripeRequest::CreateInvoiceItem).await.deref(), params)
					.instrument(tracing::info_span!("create_invoice_item"))
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to create invoice item");
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::StripeError,
							"failed to create invoice item",
						))
					})?;

				let mut invoice = stripe::Invoice::finalize(
					stripe_client.client(StripeRequest::FinalizeInvoice).await.deref(),
					&invoice.id,
					stripe::FinalizeInvoiceParams {
						auto_advance: Some(true),
					},
				)
				.instrument(tracing::info_span!("finalize_invoice"))
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to finalize invoice");
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::StripeError,
						"failed to finalize invoice",
					))
				})?;

				if invoice.status == Some(stripe::InvoiceStatus::Open) {
					let paid_invoice =
						stripe::Invoice::pay(stripe_client.client(StripeRequest::PayInvoice).await.deref(), &invoice.id)
							.instrument(tracing::info_span!("pay_invoice"))
							.await;

					match paid_invoice {
						Ok(paid_invoice) => invoice = paid_invoice,
						Err(stripe::StripeError::Stripe(e)) if e.code == Some(stripe::ErrorCode::CardDeclined) => {
							tracing::warn!(error = %e, "card declined");
						}
						Err(e) => {
							tracing::error!(error = %e, "failed to pay invoice");
							return Err(TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::StripeError,
								"failed to pay invoice",
							)));
						}
					}
				}

				let invoice_id: InvoiceId = invoice.id.into();

				let status: InvoiceStatus = invoice
					.status
					.ok_or_else(|| {
						tracing::error!("invoice status is missing");
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::StripeError,
							"invoice status is missing",
						))
					})?
					.into();

				tx.update_one(
					filter::filter! {
						Invoice {
							#[query(rename = "_id")]
							id: &invoice_id,
						}
					},
					update::update! {
						#[query(set_on_insert)]
						Invoice {
							#[query(rename = "_id")]
							id: &invoice_id,
							created_at: chrono::Utc::now(),
						},
						#[query(set)]
						Invoice {
							items: vec![price.id.clone()],
							customer_id,
							user_id: user.id,
							paypal_payment_id: &None,
							#[query(serde)]
							status,
							failed: false,
							refunded: false,
							#[query(serde)]
							disputed: &None,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					Some(UpdateOptions::builder().upsert(true).build()),
				)
				.await?;

				Ok(CreateSubscriptionPeriodResponse {
					success: true,
					invoice_id: Some(invoice_id.to_string()),
					payment_declined: invoice.status == Some(stripe::InvoiceStatus::Open),
				})
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "transaction failed");
				ApiError::internal_server_error(ApiErrorCode::TransactionError, "transaction failed")
			})?;

			Ok(status)
		} else {
			let status = transaction(global, |mut tx| async move {
				tx.insert_one(
					SubscriptionPeriod {
						id: SubscriptionPeriodId::new(),
						provider_id: None,
						product_id: price.id.clone(),
						start: create.start,
						end: create.end,
						is_trial: false,
						auto_renew: false,
						gifted_by: None,
						created_by: SubscriptionPeriodCreatedBy::System {
							reason: Some(create.reason),
						},
						subscription_id: SubscriptionId {
							product_id: product.id,
							user_id: self.id.id(),
						},
						updated_at: chrono::Utc::now(),
						search_updated_at: None,
					},
					None,
				)
				.await?;

				// TODO: create an audit log for this.

				Ok::<_, TransactionError<Infallible>>(CreateSubscriptionPeriodResponse {
					success: true,
					invoice_id: None,
					payment_declined: false,
				})
			})
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "transaction failed");
				ApiError::internal_server_error(ApiErrorCode::TransactionError, "transaction failed")
			})?;

			sub_refresh_job::refresh(
				global,
				SubscriptionId {
					product_id: product.id,
					user_id: self.id.id(),
				},
			)
			.await?;

			Ok(status)
		}
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageAny)")]
	async fn refresh_subscriptions(&self, ctx: &Context<'_>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let products = global
			.subscription_products_loader
			.load(())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load product"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "product not found"))?;

		for product in products {
			sub_refresh_job::refresh(
				global,
				SubscriptionId {
					product_id: product.id,
					user_id: self.id.id(),
				},
			)
			.await?;
		}

		Ok(true)
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
