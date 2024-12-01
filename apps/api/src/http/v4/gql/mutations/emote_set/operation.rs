use std::sync::Arc;

use async_graphql::Context;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::emote::{EmoteFlags, EmoteId};
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestId, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::emote_set::{EmoteSetEmoteFlag, EmoteSetKind};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{EmoteSetPermission, PermissionsExt, RateLimitResource, UserPermission};
use shared::database::stored_event::StoredEventEmoteModerationRequestData;
use shared::database::user::editor::{
	EditorEmoteSetPermission, EditorPermission, EditorUserPermission, UserEditorId, UserEditorState,
};
use shared::database::user::FullUserRef;
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};

use crate::dataloader::emote::EmoteByIdLoaderExt;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::{EmoteSet, EmoteSetEmote};
use crate::http::validators::{EmoteNameValidator, NameValidator};
use crate::transactions::{transaction_with_mutex, GeneralMutexKey, TransactionError};

pub struct EmoteSetOperation {
	pub emote_set: shared::database::emote_set::EmoteSet,
}

impl EmoteSetOperation {
	async fn check_perms<'a>(
		&self,
		global: &Arc<Global>,
		session: &'a Session,
		editor_perm: impl Into<EditorPermission>,
	) -> Result<FullUserRef<'a>, ApiError> {
		let mut editor_perm = editor_perm.into();
		let user = session.user()?;

		let mut target = FullUserRef::Ref(user);

		match self.emote_set.kind {
			EmoteSetKind::Global => {
				if !user.has(EmoteSetPermission::ManageGlobal) {
					return Err(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"this user does not have permission to manage global emote sets",
					));
				}

				return Ok(target);
			}
			EmoteSetKind::Special => {
				if !user.has(EmoteSetPermission::ManageSpecial) {
					return Err(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"this user does not have permission to manage special emote sets",
					));
				}

				return Ok(target);
			}
			EmoteSetKind::Personal | EmoteSetKind::Normal => {}
		}

		let owner_id = self
			.emote_set
			.owner_id
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "owner not found"))?;

		// If the person who is updating the set is not the owner, we need to load the
		// owner.
		if owner_id != user.id {
			target = FullUserRef::Owned(
				global
					.user_loader
					.load(global, owner_id)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "owner not found"))?
					.into(),
			)
		}

		let mut forbidden_msg = "you do not have permission to manage this user's emote sets";

		// If the emote set is personal, check if the owner has permission to use
		if matches!(self.emote_set.kind, EmoteSetKind::Personal) {
			if !target.has(UserPermission::UsePersonalEmoteSet) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"this user does not have permission to use personal emote sets",
				));
			}

			editor_perm = EditorUserPermission::ManagePersonalEmoteSet.into();
			forbidden_msg = "you do not have permission to manage this user's personal emote set";
		}

		if !target.has(EmoteSetPermission::Manage) && !user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, forbidden_msg));
		}

		if target.id != user.id && !user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: owner_id,
					editor_id: user.id,
				})
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(ApiErrorCode::LackingPrivileges, "you are not an editor for this user")
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(editor_perm) {
				return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, forbidden_msg));
			}
		}

		Ok(target)
	}
}

#[derive(async_graphql::InputObject, Clone)]
pub struct EmoteSetEmoteId {
	pub emote_id: EmoteId,
	pub alias: Option<String>,
}

#[derive(async_graphql::InputObject, Clone)]
pub struct AddEmote {
	pub id: EmoteSetEmoteId,
	pub zero_width: Option<bool>,
	pub override_conflicts: Option<bool>,
}

#[derive(async_graphql::InputObject, Clone)]
pub struct EmoteSetEmoteFlagsInput {
	pub zero_width: bool,
	pub override_conflicts: bool,
}

impl From<EmoteSetEmoteFlagsInput> for shared::database::emote_set::EmoteSetEmoteFlag {
	fn from(val: EmoteSetEmoteFlagsInput) -> Self {
		let mut flags = shared::database::emote_set::EmoteSetEmoteFlag::default();

		if val.zero_width {
			flags |= shared::database::emote_set::EmoteSetEmoteFlag::ZeroWidth;
		}

		if val.override_conflicts {
			flags |= shared::database::emote_set::EmoteSetEmoteFlag::OverrideConflicts;
		}

		flags
	}
}

#[async_graphql::Object]
impl EmoteSetOperation {
	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetCreate, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::create")]
	async fn create(
		&self,
		_ctx: &Context<'_>,
		#[graphql(validator(custom = "NameValidator"))] _name: String,
	) -> Result<EmoteSet, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::name")]
	async fn name(
		&self,
		_ctx: &Context<'_>,
		#[graphql(validator(custom = "NameValidator"))] _name: String,
	) -> Result<EmoteSet, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::capacity")]
	async fn capacity(
		&self,
		_ctx: &Context<'_>,
		#[graphql(validator(minimum = 1))] _capacity: i32,
	) -> Result<EmoteSet, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::add_emote")]
	async fn add_emote(&self, ctx: &Context<'_>, emote: AddEmote) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.emote_set.id).into()),
			|mut tx| async move {
				let authed_user = session.user().map_err(TransactionError::Custom)?;

				if let Some(capacity) = self.emote_set.capacity {
					if self.emote_set.emotes.len() as i32 >= capacity {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::BadRequest,
							"emote set is at capacity",
						)));
					}
				}

				let db_emote = tx
					.find_one(
						filter::filter! { shared::database::emote::Emote { #[query(rename = "_id")] id: emote.id.emote_id } },
						None,
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found"))
					})?;

				if db_emote.deleted || db_emote.merged.is_some() {
					return Err(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::BadRequest,
						"emote not found",
					)));
				}

				if db_emote.flags.contains(EmoteFlags::Private)
					&& self.emote_set.owner_id.is_none_or(|id| db_emote.owner_id != id)
				{
					return Err(TransactionError::Custom(ApiError::bad_request(
						ApiErrorCode::BadRequest,
						"emote is private",
					)));
				}

				let alias = emote.id.alias.unwrap_or_else(|| db_emote.default_name.clone());

				// This may be a problem if the emote has been deleted.
				// We should likely load all the emotes here anyways.
				// Note: we do not use the TX here because this does not really effect the
				// transaction.
				let emotes = global
					.emote_by_id_loader
					.load_many(self.emote_set.emotes.iter().map(|e| e.id))
					.await
					.map_err(|_| {
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to load emotes",
						))
					})?;

				let conflict_emote_idx = self.emote_set.emotes.iter().position(|e| e.alias == alias);

				if let Some(conflict_emote_idx) = conflict_emote_idx {
					if let Some(emote) = emotes.get(&self.emote_set.emotes[conflict_emote_idx].id) {
						if !emote.deleted {
							return Err(TransactionError::Custom(ApiError::conflict(
								ApiErrorCode::BadRequest,
								"this emote has a conflicting name",
							)));
						}
					}
				}

				if matches!(self.emote_set.kind, EmoteSetKind::Personal) {
					if db_emote.flags.contains(EmoteFlags::DeniedPersonal) {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::BadRequest,
							"emote is not allowed in personal emote sets",
						)));
					} else if !db_emote.flags.contains(EmoteFlags::ApprovedPersonal) {
						let id = EmoteModerationRequestId::new();
						let country_code = global
							.geoip()
							.and_then(|g| g.lookup(session.ip()))
							.and_then(|c| c.iso_code)
							.map(|c| c.to_string());

						let request = tx
							.find_one_and_update(
								filter::filter! {
									EmoteModerationRequest {
										#[query(serde)]
										kind: EmoteModerationRequestKind::PersonalUse,
										emote_id: db_emote.id,
									}
								},
								update::update! {
									#[query(set_on_insert)]
									EmoteModerationRequest {
										#[query(rename = "_id")]
										id,
										user_id: authed_user.id,
										#[query(serde)]
										kind: EmoteModerationRequestKind::PersonalUse,
										reason: Some("User requested to add emote to a personal set".to_string()),
										emote_id: db_emote.id,
										#[query(serde)]
										status: EmoteModerationRequestStatus::Pending,
										country_code,
										assigned_to: vec![],
										priority: authed_user
											.computed
											.permissions
											.emote_moderation_request_priority
											.unwrap_or_default(),
										search_updated_at: &None,
										updated_at: chrono::Utc::now(),
									},
								},
								FindOneAndUpdateOptions::builder()
									.upsert(true)
									.return_document(ReturnDocument::After)
									.build(),
							)
							.await?
							.ok_or_else(|| {
								TransactionError::Custom(ApiError::internal_server_error(
									ApiErrorCode::MutationError,
									"emote moderation failed to insert",
								))
							})?;

						if request.id == id {
							tx.register_event(InternalEvent {
								actor: Some(authed_user.clone()),
								session_id: session.user_session_id(),
								data: InternalEventData::EmoteModerationRequest {
									after: request,
									data: StoredEventEmoteModerationRequestData::Create,
								},
								timestamp: chrono::Utc::now(),
							})?;
						}

						let count = tx
							.count(
								filter::filter! {
									EmoteModerationRequest {
										#[query(serde)]
										kind: EmoteModerationRequestKind::PersonalUse,
										user_id: authed_user.id,
										#[query(serde)]
										status: EmoteModerationRequestStatus::Pending,
									}
								},
								None,
							)
							.await?;

						if count as i32
							> authed_user
								.computed
								.permissions
								.emote_moderation_request_limit
								.unwrap_or_default()
						{
							return Err(TransactionError::Custom(ApiError::bad_request(
								ApiErrorCode::LackingPrivileges,
								"too many pending moderation requests",
							)));
						}
					}
				}

				let mut flags = EmoteSetEmoteFlag::default();

				if emote
					.zero_width
					.unwrap_or(db_emote.flags.contains(EmoteFlags::DefaultZeroWidth))
				{
					flags |= EmoteSetEmoteFlag::ZeroWidth;
				}

				if emote.override_conflicts.unwrap_or_default() {
					flags |= EmoteSetEmoteFlag::OverrideConflicts;
				}

				let emote_set_emote = shared::database::emote_set::EmoteSetEmote {
					id: emote.id.emote_id,
					added_by_id: Some(authed_user.id),
					alias: alias.clone(),
					flags,
					added_at: chrono::Utc::now(),
					origin_set_id: None,
				};

				let update = update::Update::from(update::update! {
					#[query(set)]
					shared::database::emote_set::EmoteSet {
						emotes_changed_since_reindex: true,
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					},
				});

				let update = if let Some(conflict_idx) = conflict_emote_idx {
					update.extend_one(update::update! {
						#[query(set)]
						shared::database::emote_set::EmoteSet {
							#[query(flatten, index = "conflict_idx", serde)]
							emotes: &emote_set_emote,
						},
					})
				} else {
					update.extend_one(update::update! {
						#[query(push)]
						shared::database::emote_set::EmoteSet {
							#[query(serde)]
							emotes: &emote_set_emote,
						},
					})
				};

				let emote_set = tx
					.find_one_and_update(
						filter::filter! {
							shared::database::emote_set::EmoteSet {
								#[query(rename = "_id")]
								id: self.emote_set.id,
							}
						},
						update,
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))
					})?;

				if let Some(capacity) = emote_set.capacity {
					// Unfortunately we actually need to load all these emotes to check the deleted
					// status to determine if they contribute towards the capacity limit
					// Perhaps we could cache this in redis or something (the merge/deleted status
					// of an emote at any given time to avoid doing a DB lookup)
					let emotes = global
						.emote_by_id_loader
						.load_many_merged(emote_set.emotes.iter().map(|e| e.id))
						.await
						.map_err(|()| {
							TransactionError::Custom(ApiError::internal_server_error(
								ApiErrorCode::LoadError,
								"failed to load emotes",
							))
						})?;

					let active_emotes = emote_set.emotes.iter().filter(|e| emotes.get(e.id).is_some()).count();

					if active_emotes as i32 > capacity {
						return Err(TransactionError::Custom(ApiError::bad_request(
							ApiErrorCode::LoadError,
							"emote set is at capacity",
						)));
					}
				}

				let emote_owner = global.user_loader.load_fast(global, db_emote.owner_id).await.map_err(|_| {
					TransactionError::Custom(ApiError::internal_server_error(
						ApiErrorCode::LoadError,
						"failed to load emote owner",
					))
				})?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::EmoteSet {
						after: emote_set.clone(),
						data: InternalEventEmoteSetData::AddEmote {
							emote: Box::new(db_emote),
							emote_owner: emote_owner.map(Box::new),
							emote_set_emote,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote_set)
			},
		)
		.await;

		match res {
			Ok(emote_set) => Ok(emote_set.into()),
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
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::remove_emote")]
	async fn remove_emote(&self, ctx: &Context<'_>, id: EmoteSetEmoteId) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.emote_set.id).into()),
			|mut tx| async move {
				let authed_user = session.user().map_err(TransactionError::Custom)?;

				let old_emotes: Vec<_> = self
					.emote_set
					.emotes
					.iter()
					.enumerate()
					.filter(|(_, e)| e.id == id.emote_id && id.alias.as_ref().is_none_or(|a| e.alias == *a))
					.collect();

				if old_emotes.is_empty() {
					return Err(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::BadRequest,
						"emote not found in set",
					)));
				}

				let emote = tx
					.find_one(
						filter::filter! { shared::database::emote::Emote { #[query(rename = "_id")] id: id.emote_id } },
						None,
					)
					.await?;

				let emote_id = id.emote_id;

				let filter = if let Some(alias) = &id.alias {
					filter::filter! {
						shared::database::emote_set::EmoteSet {
							#[query(rename = "_id")]
							id: self.emote_set.id,
							#[query(flatten)]
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
								alias,
							}
						}
					}
				} else {
					filter::filter! {
						shared::database::emote_set::EmoteSet {
							#[query(rename = "_id")]
							id: self.emote_set.id,
							#[query(flatten)]
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
							}
						}
					}
				};

				let update = if let Some(alias) = &id.alias {
					update::update! {
						#[query(pull)]
						shared::database::emote_set::EmoteSet {
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
								alias: alias,
							},
						}
					}
				} else {
					update::update! {
						#[query(pull)]
						shared::database::emote_set::EmoteSet {
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
							},
						}
					}
				};

				let update = update::Update::from(update).extend_one(update::update! {
					#[query(set)]
					shared::database::emote_set::EmoteSet {
						emotes_changed_since_reindex: true,
						updated_at: chrono::Utc::now(),
						search_updated_at: &None,
					}
				});

				let emote_set = tx
					.find_one_and_update(
						filter,
						update,
						FindOneAndUpdateOptions::builder()
							.return_document(mongodb::options::ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::BadRequest,
						"emote not found in set",
					)))?;

				let emote_owner = if let Some(e) = &emote {
					global.user_loader.load_fast(global, e.owner_id).await.map_err(|_| {
						TransactionError::Custom(ApiError::internal_server_error(
							ApiErrorCode::LoadError,
							"failed to load emote owner",
						))
					})?
				} else {
					None
				};

				for (index, old_emote_set_emote) in old_emotes {
					tx.register_event(InternalEvent {
						actor: Some(authed_user.clone()),
						session_id: session.user_session_id(),
						data: InternalEventData::EmoteSet {
							after: emote_set.clone(),
							data: InternalEventEmoteSetData::RemoveEmote {
								emote: emote.clone().map(Box::new),
								emote_owner: emote_owner.clone().map(Box::new),
								emote_set_emote: old_emote_set_emote.clone(),
								index,
							},
						},
						timestamp: chrono::Utc::now(),
					})?;
				}

				Ok(emote_set)
			},
		)
		.await;

		match res {
			Ok(emote_set) => Ok(emote_set.into()),
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
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::update_emote_alias")]
	async fn update_emote_alias(
		&self,
		ctx: &Context<'_>,
		id: EmoteSetEmoteId,
		#[graphql(validator(custom = "EmoteNameValidator"))] alias: String,
	) -> Result<EmoteSetEmote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.emote_set.id).into()),
			|mut tx| async move {
				let authed_user = session.user().map_err(TransactionError::Custom)?;

				let old_emote_set_emote = self
					.emote_set
					.emotes
					.iter()
					.find(|e| e.id == id.emote_id && id.alias.as_ref().is_none_or(|a| e.alias == *a))
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set"))
					})?;

				let emote = tx
					.find_one(
						filter::filter! { shared::database::emote::Emote { #[query(rename = "_id")] id: id.emote_id } },
						None,
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found"))
					})?;

				if emote.deleted || emote.merged.is_some() {
					return Err(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::BadRequest,
						"emote not found",
					)));
				}

				if self.emote_set.emotes.iter().any(|e| e.alias == alias) {
					return Err(TransactionError::Custom(ApiError::conflict(
						ApiErrorCode::BadRequest,
						"emote name conflict",
					)));
				}

				let emote_id = id.emote_id;

				let filter = if let Some(alias) = id.alias {
					filter::filter! {
						shared::database::emote_set::EmoteSet {
							#[query(rename = "_id")]
							id: self.emote_set.id,
							#[query(flatten)]
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
								alias,
							}
						}
					}
				} else {
					filter::filter! {
						shared::database::emote_set::EmoteSet {
							#[query(rename = "_id")]
							id: self.emote_set.id,
							#[query(flatten)]
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
							}
						}
					}
				};

				let emote_set = tx
					.find_one_and_update(
						filter,
						update::update! {
							#[query(set)]
							shared::database::emote_set::EmoteSet {
								#[query(flatten, index = "$")]
								emotes: shared::database::emote_set::EmoteSetEmote {
									alias,
								},
								emotes_changed_since_reindex: true,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
						},
						FindOneAndUpdateOptions::builder()
							.return_document(mongodb::options::ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set"))
					})?;

				let emote_set_emote = emote_set.emotes.iter().find(|e| e.id == id.emote_id).ok_or_else(|| {
					TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set"))
				})?;

				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::EmoteSet {
						after: emote_set.clone(),
						data: InternalEventEmoteSetData::RenameEmote {
							emote: Box::new(emote),
							emote_set_emote: emote_set_emote.clone(),
							old_alias: old_emote_set_emote.alias.clone(),
						},
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote_set_emote.clone())
			},
		)
		.await;

		match res {
			Ok(emote_set_emote) => Ok(emote_set_emote.into()),
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
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::update_emote_flags")]
	async fn update_emote_flags(
		&self,
		ctx: &Context<'_>,
		id: EmoteSetEmoteId,
		flags: EmoteSetEmoteFlagsInput,
	) -> Result<EmoteSetEmote, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.emote_set.id).into()),
			|mut tx| async move {
				session.user().map_err(TransactionError::Custom)?;

				let emote = tx
					.find_one(
						filter::filter! { shared::database::emote::Emote { #[query(rename = "_id")] id: id.emote_id } },
						None,
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found"))
					})?;

				if emote.deleted || emote.merged.is_some() {
					return Err(TransactionError::Custom(ApiError::not_found(
						ApiErrorCode::BadRequest,
						"emote not found",
					)));
				}

				let emote_id = id.emote_id;

				let new_flags: EmoteSetEmoteFlag = flags.into();

				let filter = if let Some(alias) = id.alias {
					filter::filter! {
						shared::database::emote_set::EmoteSet {
							#[query(rename = "_id")]
							id: self.emote_set.id,
							#[query(flatten)]
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
								alias,
							}
						}
					}
				} else {
					filter::filter! {
						shared::database::emote_set::EmoteSet {
							#[query(rename = "_id")]
							id: self.emote_set.id,
							#[query(flatten)]
							emotes: shared::database::emote_set::EmoteSetEmote {
								id: emote_id,
							}
						}
					}
				};

				let emote_set = tx
					.find_one_and_update(
						filter,
						update::update! {
							#[query(set)]
							shared::database::emote_set::EmoteSet {
								#[query(flatten, index = "$")]
								emotes: shared::database::emote_set::EmoteSetEmote {
									#[query(serde)]
									flags: new_flags,
								},
								emotes_changed_since_reindex: true,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							},
						},
						FindOneAndUpdateOptions::builder()
							.return_document(mongodb::options::ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set"))
					})?;

				let emote_set_emote = emote_set.emotes.iter().find(|e| e.id == id.emote_id).ok_or_else(|| {
					TransactionError::Custom(ApiError::not_found(ApiErrorCode::BadRequest, "emote not found in set"))
				})?;

				Ok(emote_set_emote.clone())
			},
		)
		.await;

		match res {
			Ok(emote_set_emote) => Ok(emote_set_emote.into()),
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
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::delete")]
	async fn delete(&self, _ctx: &Context<'_>) -> Result<bool, ApiError> {
		Err(ApiError::not_implemented(ApiErrorCode::BadRequest, "not implemented"))
	}
}
