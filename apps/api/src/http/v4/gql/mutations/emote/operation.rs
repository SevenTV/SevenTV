use std::sync::Arc;

use async_graphql::Context;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::emote::{EmoteFlags, EmoteId};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{
	EmoteModerationRequestPermission, EmotePermission, PermissionsExt, RateLimitResource,
};
use shared::database::stored_event::StoredEventEmoteData;
use shared::database::user::editor::{EditorEmotePermission, UserEditorId, UserEditorState};
use shared::database::user::UserId;
use shared::event::{InternalEvent, InternalEventData};

use super::EmoteFlagsInput;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::Emote;
use crate::http::validators::EmoteNameValidator;
use crate::transactions::{transaction_with_mutex, GeneralMutexKey, TransactionError};

pub struct EmoteOperation {
	pub emote: shared::database::emote::Emote,
}

impl EmoteOperation {
	async fn check_permission(
		&self,
		global: &Arc<Global>,
		session: &Session,
		requires_manage_any: bool,
		permission: EmotePermission,
	) -> Result<(), ApiError> {
		let authed_user = session.user()?;

		if requires_manage_any && !authed_user.has(EmotePermission::ManageAny) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you do not have permission to edit this emote",
			));
		}

		if authed_user.id != self.emote.owner_id && !authed_user.has(EmotePermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: self.emote.owner_id,
					editor_id: authed_user.id,
				})
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"you do not have permission to edit this emote",
					)
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has_emote(EditorEmotePermission::Manage) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to edit this emote",
				));
			}
		}

		if !authed_user.has(permission) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you do not have permission to edit this emote",
			));
		}

		Ok(())
	}

	async fn check_edit_permission(
		&self,
		global: &Arc<Global>,
		session: &Session,
		requires_manage_any: bool,
	) -> Result<(), ApiError> {
		self.check_permission(global, session, requires_manage_any, EmotePermission::Edit)
			.await
	}
}

#[async_graphql::Object]
impl EmoteOperation {
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	#[tracing::instrument(skip_all, name = "EmoteOperation::name")]
	async fn name(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(custom = "EmoteNameValidator"))] name: String,
	) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		self.check_edit_permission(global, session, false).await?;

		if name == self.emote.default_name {
			return Ok(Emote::from_db(self.emote.clone(), &global.config.api.cdn_origin));
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::Emote(self.emote.id).into()),
			|mut tx| async move {
				let emote = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::emote::Emote {
								#[query(rename = "_id")]
								id: self.emote.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::emote::Emote {
								default_name: &name,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))
					.map_err(TransactionError::Custom)?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::Emote {
						after: emote.clone(),
						data: StoredEventEmoteData::ChangeName {
							old: self.emote.default_name.clone(),
							new: name,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote)
			},
		)
		.await;

		match res {
			Ok(emote) => Ok(Emote::from_db(emote, &global.config.api.cdn_origin)),
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

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	#[tracing::instrument(skip_all, name = "EmoteOperation::flags")]
	async fn flags(&self, ctx: &Context<'_>, flags: EmoteFlagsInput) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let flags = flags.apply_to(self.emote.flags);

		let admin_flags = [
			EmoteFlags::PublicListed,
			EmoteFlags::Nsfw,
			EmoteFlags::ApprovedPersonal,
			EmoteFlags::DeniedPersonal,
			EmoteFlags::Animated,
		];

		let requires_manage_any = admin_flags
			.iter()
			.any(|&flag| flags.contains(flag) != self.emote.flags.contains(flag));

		self.check_edit_permission(global, session, requires_manage_any).await?;

		if flags == self.emote.flags {
			return Ok(Emote::from_db(self.emote.clone(), &global.config.api.cdn_origin));
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::Emote(self.emote.id).into()),
			|mut tx| async move {
				// Resolve emote moderation request if user has permission to manage them
				if authed_user.has(EmoteModerationRequestPermission::Manage) {
					if !self.emote.flags.contains(EmoteFlags::PublicListed) && flags.contains(EmoteFlags::PublicListed) {
						tx.find_one_and_update(
							filter::filter! {
								EmoteModerationRequest {
									emote_id: self.emote.id,
									#[query(serde)]
									kind: EmoteModerationRequestKind::PublicListing,
									#[query(serde)]
									status: EmoteModerationRequestStatus::Pending,
								}
							},
							update::update! {
								#[query(set)]
								EmoteModerationRequest {
									#[query(serde)]
									status: EmoteModerationRequestStatus::Approved,
									updated_at: chrono::Utc::now(),
									search_updated_at: &None,
								}
							},
							None,
						)
						.await?;
					}

					if !self.emote.flags.contains(EmoteFlags::ApprovedPersonal)
						&& flags.contains(EmoteFlags::ApprovedPersonal)
					{
						tx.find_one_and_update(
							filter::filter! {
								EmoteModerationRequest {
									emote_id: self.emote.id,
									#[query(serde)]
									kind: EmoteModerationRequestKind::PersonalUse,
									#[query(serde)]
									status: EmoteModerationRequestStatus::Pending,
								}
							},
							update::update! {
								#[query(set)]
								EmoteModerationRequest {
									#[query(serde)]
									status: EmoteModerationRequestStatus::Approved,
									updated_at: chrono::Utc::now(),
									search_updated_at: &None,
								}
							},
							None,
						)
						.await?;
					}

					if !self.emote.flags.contains(EmoteFlags::DeniedPersonal) && flags.contains(EmoteFlags::DeniedPersonal) {
						tx.find_one_and_update(
							filter::filter! {
								EmoteModerationRequest {
									emote_id: self.emote.id,
									#[query(serde)]
									kind: EmoteModerationRequestKind::PersonalUse,
									#[query(serde)]
									status: EmoteModerationRequestStatus::Pending,
								}
							},
							update::update! {
								#[query(set)]
								EmoteModerationRequest {
									#[query(serde)]
									status: EmoteModerationRequestStatus::Denied,
									updated_at: chrono::Utc::now(),
									search_updated_at: &None,
								}
							},
							None,
						)
						.await?;
					}
				}

				let emote = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::emote::Emote {
								#[query(rename = "_id")]
								id: self.emote.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::emote::Emote {
								flags: flags,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))
					.map_err(TransactionError::Custom)?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::Emote {
						after: emote.clone(),
						data: StoredEventEmoteData::ChangeFlags {
							old: self.emote.flags,
							new: flags,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote)
			},
		)
		.await;

		match res {
			Ok(emote) => Ok(Emote::from_db(emote, &global.config.api.cdn_origin)),
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

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	#[tracing::instrument(skip_all, name = "EmoteOperation::owner")]
	async fn owner(&self, ctx: &Context<'_>, owner_id: UserId) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		self.check_edit_permission(global, session, true).await?;

		if owner_id == self.emote.owner_id {
			return Ok(Emote::from_db(self.emote.clone(), &global.config.api.cdn_origin));
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::Emote(self.emote.id).into()),
			|mut tx| async move {
				let emote = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::emote::Emote {
								#[query(rename = "_id")]
								id: self.emote.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::emote::Emote {
								owner_id: owner_id,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))
					.map_err(TransactionError::Custom)?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::Emote {
						after: emote.clone(),
						data: StoredEventEmoteData::ChangeOwner {
							old: self.emote.owner_id,
							new: owner_id,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote)
			},
		)
		.await;

		match res {
			Ok(emote) => Ok(Emote::from_db(emote, &global.config.api.cdn_origin)),
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

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	#[tracing::instrument(skip_all, name = "EmoteOperation::tags")]
	async fn tags(&self, ctx: &Context<'_>, tags: Vec<String>) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		self.check_edit_permission(global, session, false).await?;

		if tags == self.emote.tags {
			return Ok(Emote::from_db(self.emote.clone(), &global.config.api.cdn_origin));
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::Emote(self.emote.id).into()),
			|mut tx| async move {
				let emote = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::emote::Emote {
								#[query(rename = "_id")]
								id: self.emote.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::emote::Emote {
								tags: &tags,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))
					.map_err(TransactionError::Custom)?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::Emote {
						after: emote.clone(),
						data: StoredEventEmoteData::ChangeTags {
							old: self.emote.tags.clone(),
							new: tags,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote)
			},
		)
		.await;

		match res {
			Ok(emote) => Ok(Emote::from_db(emote, &global.config.api.cdn_origin)),
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

	#[graphql(
		guard = "PermissionGuard::one(EmotePermission::Merge).and(RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteOperation::merge")]
	async fn merge(&self, _ctx: &Context<'_>, _with: EmoteId) -> Result<Emote, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	#[tracing::instrument(skip_all, name = "EmoteOperation::delete")]
	async fn delete(&self, ctx: &Context<'_>) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		self.check_permission(global, session, false, EmotePermission::Delete).await?;

		if self.emote.deleted {
			return Ok(Emote::from_db(self.emote.clone(), &global.config.api.cdn_origin));
		}

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::Emote(self.emote.id).into()),
			|mut tx| async move {
				let emote = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::emote::Emote {
								#[query(rename = "_id")]
								id: self.emote.id,
							}
						},
						update::update! {
							#[query(set)]
							shared::database::emote::Emote {
								deleted: true,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote not found"))
					.map_err(TransactionError::Custom)?;

				tx.update(
					filter::filter! {
						EmoteModerationRequest {
							emote_id: self.emote.id,
							#[query(serde)]
							status: EmoteModerationRequestStatus::Pending,
						}
					},
					update::update! {
						#[query(set)]
						EmoteModerationRequest {
							#[query(serde)]
							status: EmoteModerationRequestStatus::EmoteDeleted,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::Emote {
						after: emote.clone(),
						data: StoredEventEmoteData::Delete,
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote)
			},
		)
		.await;

		return match res {
			Ok(emote) => Ok(Emote::from_db(emote, &global.config.api.cdn_origin)),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		};
	}
}
