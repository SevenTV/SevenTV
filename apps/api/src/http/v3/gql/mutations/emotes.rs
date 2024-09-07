use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::emote::{Emote as DbEmote, EmoteFlags, EmoteMerged};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{EmotePermission, PermissionsExt};
use shared::database::stored_event::StoredEventEmoteData;
use shared::database::user::editor::{EditorEmotePermission, UserEditorId, UserEditorState};
use shared::event::{InternalEvent, InternalEventData};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::EmoteFlagsModel;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::emote::Emote;
use crate::http::v3::validators::{NameValidator, TagsValidator};
use crate::transactions::{with_transaction, TransactionError};

#[derive(Default)]
pub struct EmotesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmotesMutation {
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<EmoteOps, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote = global
			.emote_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		Ok(EmoteOps { id, emote: emote })
	}
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteOps {
	id: GqlObjectId,
	#[graphql(skip)]
	emote: DbEmote,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteOps {
	async fn update<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		params: EmoteUpdate,
		_reason: Option<String>,
	) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = auth_session.user(global).await?;

		if user.id != self.emote.owner_id && !user.has(EmotePermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: self.emote.owner_id,
					editor_id: user.id,
				})
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::FORBIDDEN)?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has_emote(EditorEmotePermission::Manage) {
				return Err(ApiError::FORBIDDEN);
			}
		}

		if params.deleted.is_some_and(|d| d) {
			if !user.has(EmotePermission::Delete) {
				return Err(ApiError::FORBIDDEN);
			}

			// TODO: don't allow deletion of emotes that are in use
			// TODO: delete files from s3

			let res = with_transaction(global, |mut tx| async move {
				let emote = tx
					.find_one_and_delete(
						filter::filter! {
							DbEmote {
								#[query(rename = "_id")]
								id: self.id.id(),
							}
						},
						None,
					)
					.await?
					.ok_or(ApiError::NOT_FOUND)
					.map_err(TransactionError::custom)?;

				tx.register_event(InternalEvent {
					actor: Some(user.clone()),
					data: InternalEventData::Emote {
						after: emote.clone(),
						data: StoredEventEmoteData::Delete,
					},
					timestamp: chrono::Utc::now(),
				})?;

				Ok(emote)
			})
			.await;

			match res {
				Ok(emote) => Ok(Emote::from_db(global, emote)),
				Err(TransactionError::Custom(e)) => Err(e),
				Err(e) => {
					tracing::error!(error = %e, "transaction failed");
					Err(ApiError::INTERNAL_SERVER_ERROR)
				}
			}
		} else {
			if !user.has(EmotePermission::Edit) {
				return Err(ApiError::FORBIDDEN);
			}

			let res = with_transaction(global, |mut tx| async move {
				let new_default_name = params.name.or(params.version_name);

				let mut flags = self.emote.flags;

				if let Some(input_flags) = params.flags {
					if input_flags.contains(EmoteFlagsModel::Private) {
						flags |= EmoteFlags::Private;
						flags &= !EmoteFlags::PublicListed;
					} else {
						flags &= !EmoteFlags::Private;
						flags |= EmoteFlags::PublicListed;
					}

					if input_flags.contains(EmoteFlagsModel::ZeroWidth) {
						flags |= EmoteFlags::DefaultZeroWidth;
					} else {
						flags &= !EmoteFlags::DefaultZeroWidth;
					}
				}

				// changing visibility and owner requires manage any perms
				let new_owner_id = if user.has(EmotePermission::ManageAny) {
					if let Some(listed) = params.listed {
						if listed {
							flags |= EmoteFlags::PublicListed;
							flags &= !EmoteFlags::Private;
						} else {
							flags &= !EmoteFlags::PublicListed;
							flags |= EmoteFlags::Private;
						}
					}

					if let Some(personal_use) = params.personal_use {
						if personal_use {
							flags |= EmoteFlags::ApprovedPersonal;
							flags &= !EmoteFlags::DeniedPersonal;
						} else {
							flags &= !EmoteFlags::ApprovedPersonal;
							flags |= EmoteFlags::DeniedPersonal;
						}
					}

					params.owner_id.map(|id| id.id())
				} else {
					None
				};

				let new_flags = (flags != self.emote.flags).then_some(flags);

				let emote = tx
					.find_one_and_update(
						filter::filter! {
							DbEmote {
								#[query(rename = "_id")]
								id: self.id.id(),
							}
						},
						update::update! {
							#[query(set)]
							DbEmote {
								#[query(optional)]
								default_name: new_default_name.as_ref(),
								#[query(optional)]
								owner_id: new_owner_id,
								#[query(optional)]
								flags: new_flags,
								#[query(optional)]
								tags: params.tags.as_ref(),
								updated_at: chrono::Utc::now(),
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or(ApiError::NOT_FOUND)
					.map_err(TransactionError::custom)?;

				if let Some(new_default_name) = new_default_name {
					tx.register_event(InternalEvent {
						actor: Some(user.clone()),
						data: InternalEventData::Emote {
							after: emote.clone(),
							data: StoredEventEmoteData::ChangeName {
								old: self.emote.default_name.clone(),
								new: new_default_name,
							},
						},
						timestamp: chrono::Utc::now(),
					})?;
				}

				if let Some(new_owner_id) = new_owner_id {
					tx.register_event(InternalEvent {
						actor: Some(user.clone()),
						data: InternalEventData::Emote {
							after: emote.clone(),
							data: StoredEventEmoteData::ChangeOwner {
								old: self.emote.owner_id,
								new: new_owner_id,
							},
						},
						timestamp: chrono::Utc::now(),
					})?;
				}

				if let Some(new_flags) = new_flags {
					tx.register_event(InternalEvent {
						actor: Some(user.clone()),
						data: InternalEventData::Emote {
							after: emote.clone(),
							data: StoredEventEmoteData::ChangeFlags {
								old: self.emote.flags,
								new: new_flags,
							},
						},
						timestamp: chrono::Utc::now(),
					})?;
				}

				if let Some(new_tags) = params.tags {
					tx.register_event(InternalEvent {
						actor: Some(user.clone()),
						data: InternalEventData::Emote {
							after: emote.clone(),
							data: StoredEventEmoteData::ChangeTags {
								old: self.emote.tags.clone(),
								new: new_tags,
							},
						},
						timestamp: Utc::now(),
					})?;
				}

				Ok(emote)
			})
			.await;

			match res {
				Ok(emote) => Ok(Emote::from_db(global, emote)),
				Err(TransactionError::Custom(e)) => Err(e),
				Err(e) => {
					tracing::error!(error = %e, "transaction failed");
					Err(ApiError::INTERNAL_SERVER_ERROR)
				}
			}
		}
	}

	#[graphql(guard = "PermissionGuard::one(EmotePermission::Merge)")]
	async fn merge<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		target_id: GqlObjectId,
		_reason: Option<String>,
	) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let res = with_transaction(global, |mut tx| async move {
			let emote = tx
				.find_one_and_update(
					filter::filter! {
						DbEmote {
							#[query(rename = "_id")]
							id: self.id.id(),
						}
					},
					update::update! {
						#[query(set)]
						DbEmote {
							#[query(serde)]
							merged: EmoteMerged {
								target_id: target_id.id(),
								at: chrono::Utc::now(),
							},
							updated_at: chrono::Utc::now(),
						}
					},
					None,
				)
				.await?
				.ok_or(ApiError::NOT_FOUND)
				.map_err(TransactionError::custom)?;

			tx.register_event(InternalEvent {
				actor: Some(auth_session.user(global).await.map_err(TransactionError::custom)?.clone()),
				data: InternalEventData::Emote {
					after: emote.clone(),
					data: StoredEventEmoteData::Merge {
						new_emote_id: target_id.id(),
					},
				},
				timestamp: chrono::Utc::now(),
			})?;

			// TODO: schedule emote merge job

			Ok(emote)
		})
		.await;

		match res {
			Ok(emote) => Ok(Emote::from_db(global, emote)),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
	}

	#[graphql(guard = "PermissionGuard::one(EmotePermission::Admin)")]
	async fn rerun(&self) -> Result<Option<Emote>, ApiError> {
		// will be left unimplemented
		Err(ApiError::NOT_IMPLEMENTED)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteUpdate {
	#[graphql(validator(custom = "NameValidator"))]
	name: Option<String>,
	#[graphql(validator(custom = "NameValidator"))]
	version_name: Option<String>,
	#[graphql(validator(custom = "NameValidator"))]
	version_description: Option<String>,
	flags: Option<EmoteFlagsModel>,
	owner_id: Option<GqlObjectId>,
	#[graphql(validator(custom = "TagsValidator"))]
	tags: Option<Vec<String>>,
	listed: Option<bool>,
	personal_use: Option<bool>,
	deleted: Option<bool>,
}
