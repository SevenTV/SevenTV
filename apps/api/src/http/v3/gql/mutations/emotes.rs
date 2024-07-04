use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::bson::doc;
use mongodb::options::ReturnDocument;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteData, AuditLogId};
use shared::database::emote::{Emote as DbEmote, EmoteFlags};
use shared::database::role::permissions::{EmotePermission, PermissionsExt};
use shared::database::user::editor::{EditorEmotePermission, UserEditorState};
use shared::database::Collection;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::EmoteFlagsModel;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::emote::Emote;

#[derive(Default)]
pub struct EmotesMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmotesMutation {
	async fn emote<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<EmoteOps, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote = global
			.emote_by_id_loader()
			.load(id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
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
				.user_editor_by_id_loader()
				.load((self.emote.owner_id, user.id))
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::FORBIDDEN)?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has_emote(EditorEmotePermission::Manage) {
				return Err(ApiError::FORBIDDEN);
			}
		}

		let mut session = global.mongo().start_session().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		if params.deleted.is_some_and(|d| d) {
			if !user.has(EmotePermission::Delete) {
				return Err(ApiError::FORBIDDEN);
			}

			// TODO: don't allow deletion of emotes that are in use

			let emote = shared::database::emote::Emote::collection(global.db())
				.find_one_and_delete(doc! { "_id": self.id.0 })
				.session(&mut session)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::NOT_FOUND)?;

			AuditLog::collection(global.db())
				.insert_one(AuditLog {
					id: AuditLogId::new(),
					actor_id: Some(user.id),
					data: AuditLogData::Emote {
						target_id: emote.id,
						data: AuditLogEmoteData::Delete,
					},
				})
				.session(&mut session)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to insert audit log");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

			// TODO: delete files from s3

			Ok(Emote::from_db(global, emote))
		} else {
			if !user.has(EmotePermission::Edit) {
				return Err(ApiError::FORBIDDEN);
			}

			let mut update = doc! {};

			if let Some(name) = params.name.or(params.version_name) {
				update.insert("default_name", &name);

				AuditLog::collection(global.db())
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(user.id),
						data: AuditLogData::Emote {
							target_id: self.id.id(),
							data: AuditLogEmoteData::ChangeName {
								old: self.emote.default_name.clone(),
								new: name,
							},
						},
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}

			if let Some(tags) = params.tags {
				update.insert("tags", &tags);

				AuditLog::collection(global.db())
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(user.id),
						data: AuditLogData::Emote {
							target_id: self.id.id(),
							data: AuditLogEmoteData::ChangeTags { old: self.emote.tags.clone(), new: tags },
						},
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}

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
			if user.has(EmotePermission::ManageAny) {
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

				if let Some(owner_id) = params.owner_id {
					update.insert("owner_id", owner_id.0);

					AuditLog::collection(global.db())
						.insert_one(AuditLog {
							id: AuditLogId::new(),
							actor_id: Some(user.id),
							data: AuditLogData::Emote {
								target_id: self.id.id(),
								data: AuditLogEmoteData::ChangeOwner {
									old: self.emote.owner_id,
									new: owner_id.id(),
								},
							},
						})
						.session(&mut session)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to insert audit log");
							ApiError::INTERNAL_SERVER_ERROR
						})?;
				}
			}

			if flags != self.emote.flags {
				update.insert("flags", flags.bits());

				AuditLog::collection(global.db())
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(user.id),
						data: AuditLogData::Emote {
							target_id: self.id.id(),
							data: AuditLogEmoteData::ChangeFlags {
								old: self.emote.flags,
								new: flags,
							},
						},
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}

			let emote = shared::database::emote::Emote::collection(global.db())
				.find_one_and_update(doc! { "_id": self.id.0 }, doc! { "$set": update })
				.return_document(ReturnDocument::After)
				.session(&mut session)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to update emote");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.ok_or(ApiError::NOT_FOUND)?;

			session.commit_transaction().await.map_err(|e| {
				tracing::error!(error = %e, "failed to commit transaction");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

			Ok(Emote::from_db(global, emote))
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

		let mut session = global.mongo().start_session().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let emote = shared::database::emote::Emote::collection(global.db())
			.find_one_and_update(
				doc! { "_id": self.id.0 },
				doc! {
					"$set": {
						"merged.target_id": target_id.0,
					},
					"$currentDate": {
						"merged.at": { "$type": "date" },
					},
				},
			)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update emote");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::NOT_FOUND)?;

		AuditLog::collection(global.db())
			.insert_one(AuditLog {
				id: AuditLogId::new(),
				actor_id: Some(auth_session.user_id()),
				data: AuditLogData::Emote {
					target_id: self.id.id(),
					data: AuditLogEmoteData::Merge { new_emote_id: target_id.id() },
				},
			})
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert audit log");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		// TODO: schedule emote merge job

		session.commit_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;
		
		Ok(Emote::from_db(global, emote))
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
	name: Option<String>,
	version_name: Option<String>,
	version_description: Option<String>,
	flags: Option<EmoteFlagsModel>,
	owner_id: Option<GqlObjectId>,
	tags: Option<Vec<String>>,
	listed: Option<bool>,
	personal_use: Option<bool>,
	deleted: Option<bool>,
}
