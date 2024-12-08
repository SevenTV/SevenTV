use std::sync::Arc;

use async_graphql::Context;
use shared::database::emote_set::{EmoteSetId, EmoteSetKind};
use shared::database::queries::filter;
use shared::database::role::permissions::{EmoteSetPermission, PermissionsExt, RateLimitResource};
use shared::database::user::editor::{EditorEmoteSetPermission, UserEditorId, UserEditorState};
use shared::database::user::UserId;
use shared::event::{InternalEvent, InternalEventData, InternalEventEmoteSetData};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::{PermissionGuard, RateLimitGuard};
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::EmoteSet;
use crate::http::validators::{NameValidator, TagsValidator};
use crate::transactions::{transaction, TransactionError};

mod operation;

#[derive(Default)]
pub struct EmoteSetMutation;

#[async_graphql::Object]
impl EmoteSetMutation {
	#[tracing::instrument(skip_all, name = "EmoteSetMutation::emote_set")]
	async fn emote_set(&self, ctx: &Context<'_>, id: EmoteSetId) -> Result<operation::EmoteSetOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))?;

		Ok(operation::EmoteSetOperation { emote_set })
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetCreate, 1))"
	)]
	#[tracing::instrument(skip_all, name = "EmoteSetOperation::create")]
	async fn create(
		&self,
		ctx: &Context<'_>,
		#[graphql(validator(custom = "NameValidator"))] name: String,
		#[graphql(validator(custom = "TagsValidator"))] tags: Vec<String>,
		owner_id: Option<UserId>,
	) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let owner_id = owner_id.unwrap_or(authed_user.id);

		let owner = if owner_id == authed_user.id {
			None
		} else {
			Some(
				global
					.user_loader
					.load(global, owner_id)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?,
			)
		};

		let target = owner.as_ref().unwrap_or(authed_user);

		if !target.has(EmoteSetPermission::Manage) && !authed_user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"this user does not have permission to create emote sets",
			));
		}

		if target.id != authed_user.id && !authed_user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: owner_id,
					editor_id: authed_user.id,
				})
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(ApiErrorCode::LackingPrivileges, "you are not an editor for this user")
				})?;

			if editor.state != UserEditorState::Accepted
				|| !editor.permissions.has_emote_set(EditorEmoteSetPermission::Create)
			{
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"you do not have permission to create emote sets for this user",
				));
			}
		}

		let capacity = target.computed.permissions.emote_set_capacity.unwrap_or_default().max(0);

		if capacity == 0 {
			return Err(ApiError::bad_request(
				ApiErrorCode::LackingPrivileges,
				"maximum emote set capacity is 0, cannot create emote set",
			));
		}

		let res = transaction(global, |mut tx| async move {
			let emote_set_count = tx
				.count(
					filter::filter! {
						shared::database::emote_set::EmoteSet {
							owner_id: Some(owner_id),
						}
					},
					None,
				)
				.await?;

			if emote_set_count >= (target.computed.permissions.emote_set_limit.unwrap_or(0).max(0) as u64) {
				return Err(TransactionError::Custom(ApiError::bad_request(
					ApiErrorCode::LackingPrivileges,
					"maximum emote set limit reached",
				)));
			}

			let emote_set = shared::database::emote_set::EmoteSet {
				id: Default::default(),
				owner_id: Some(owner_id),
				name,
				capacity: Some(capacity),
				description: None,
				emotes: vec![],
				kind: EmoteSetKind::Normal,
				origin_config: None,
				tags,
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
				emotes_changed_since_reindex: false,
			};

			tx.insert_one::<shared::database::emote_set::EmoteSet>(&emote_set, None)
				.await?;

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::EmoteSet {
					after: emote_set.clone(),
					data: InternalEventEmoteSetData::Create,
				},
				timestamp: chrono::Utc::now(),
			})?;

			Ok(emote_set)
		})
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
}
