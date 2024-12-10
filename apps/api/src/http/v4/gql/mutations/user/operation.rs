use std::sync::Arc;

use async_graphql::Context;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::badge::BadgeId;
use shared::database::emote_set::{EmoteSetId, EmoteSetKind};
use shared::database::paint::PaintId;
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{PermissionsExt, RateLimitResource, UserPermission};
use shared::database::user::editor::{EditorUserPermission, UserEditorId, UserEditorState};
use shared::event::{InternalEvent, InternalEventData, InternalEventUserData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::RateLimitGuard;
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::User;
use crate::transactions::{transaction_with_mutex, GeneralMutexKey, TransactionError};

pub struct UserOperation {
	pub user: shared::database::user::User,
}

#[async_graphql::Object]
impl UserOperation {
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::UserChangeConnections, 1)")]
	#[tracing::instrument(skip_all, name = "UserOperation::active_emote_set")]
	async fn active_emote_set(&self, ctx: &Context<'_>, emote_set_id: Option<EmoteSetId>) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| {
			crate::http::error::ApiError::internal_server_error(
				crate::http::error::ApiErrorCode::MissingContext,
				"missing global data",
			)
		})?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.user.id && !authed_user.has(UserPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id: self.user.id,
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
			Some(GeneralMutexKey::User(self.user.id).into()),
			|mut tx| async move {
				// check if set exists
				let emote_set = if let Some(emote_set_id) = emote_set_id {
					let emote_set = global
						.emote_set_by_id_loader
						.load(emote_set_id)
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

				let user = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::user::User {
								#[query(rename = "_id")]
								id: self.user.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::user::User {
								#[query(flatten)]
								style: shared::database::user::UserStyle {
									active_emote_set_id: emote_set_id,
								},
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))
					})?;

				let old = if let Some(set_id) = self.user.style.active_emote_set_id {
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
							new: emote_set.map(Box::new),
						},
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(user)
			},
		)
		.await;

		match res {
			Ok(user) => {
				let full_user = global
					.user_loader
					.load_fast_user(global, user)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

				Ok(full_user.into())
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
	#[tracing::instrument(skip_all, name = "UserOperation::badge")]
	async fn active_badge(&self, ctx: &Context<'_>, badge_id: Option<BadgeId>) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.user.id && !authed_user.has(UserPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id: self.user.id,
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
			.load(global, self.user.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		// check if user has paint
		if badge_id.is_some_and(|id| !user.computed.entitlements.badges.contains(&id)) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LoadError,
				"you do not have permission to use this badge",
			));
		}

		if user.style.active_badge_id == badge_id {
			return Ok(user.into());
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::User(self.user.id).into()),
			|mut tx| async move {
				let new = if let Some(id) = badge_id {
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

				let user = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::user::User {
								#[query(rename = "_id")]
								id: user.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::user::User {
								#[query(flatten)]
								style: shared::database::user::UserStyle {
									active_badge_id: badge_id,
								},
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))
					})?;

				Ok(user)
			},
		)
		.await;

		match res {
			Ok(user) => {
				let full_user = global
					.user_loader
					.load_fast_user(global, user)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

				Ok(full_user.into())
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
	#[tracing::instrument(skip_all, name = "UserOperation::paint")]
	async fn active_paint(&self, ctx: &Context<'_>, paint_id: Option<PaintId>) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.user.id && !authed_user.has(UserPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					editor_id: authed_user.id,
					user_id: self.user.id,
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
			.load(global, self.user.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		// check if user has paint
		if paint_id.is_some_and(|id| !user.computed.entitlements.paints.contains(&id)) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LoadError,
				"you do not have permission to use this paint",
			));
		}

		if user.style.active_paint_id == paint_id {
			return Ok(user.into());
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::User(self.user.id).into()),
			|mut tx| async move {
				let new = if let Some(id) = paint_id {
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
							"failed to load paint",
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

				let user = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::user::User {
								#[query(rename = "_id")]
								id: user.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::user::User {
								#[query(flatten)]
								style: shared::database::user::UserStyle {
									active_paint_id: paint_id,
								},
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "user not found"))
					})?;

				Ok(user)
			},
		)
		.await;

		match res {
			Ok(user) => {
				let full_user = global
					.user_loader
					.load_fast_user(global, user)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

				Ok(full_user.into())
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
}
