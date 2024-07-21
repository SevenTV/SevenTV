use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::options::ReturnDocument;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogEmoteData, AuditLogId};
use shared::database::emote::{Emote as DbEmote, EmoteFlags, EmoteMerged};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{EmotePermission, PermissionsExt};
use shared::database::user::editor::{EditorEmotePermission, UserEditorId, UserEditorState};
use shared::database::MongoCollection;
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, EventType};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{EmoteFlagsModel, UserPartialModel};

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

		let mut changes = vec![];
		let mut nested_changes = vec![];

		let mut session = global.mongo.start_session().await.map_err(|e| {
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

			let emote = shared::database::emote::Emote::collection(&global.db)
				.find_one_and_delete(filter::filter! {
					DbEmote {
						#[filter(rename = "_id")]
						id: self.id.id(),
					}
				})
				.session(&mut session)
				.await
				.map_err(|err| {
					tracing::error!(error = %err, "failed to delete emote");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.ok_or(ApiError::NOT_FOUND)?;

			AuditLog::collection(&global.db)
				.insert_one(AuditLog {
					id: AuditLogId::new(),
					actor_id: Some(user.id),
					data: AuditLogData::Emote {
						target_id: emote.id,
						data: AuditLogEmoteData::Delete,
					},
					updated_at: chrono::Utc::now(),
					search_updated_at: None,
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

				AuditLog::collection(&global.db)
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(user.id),
						data: AuditLogData::Emote {
							target_id: self.id.id(),
							data: AuditLogEmoteData::ChangeName {
								old: self.emote.default_name.clone(),
								new: name.clone(),
							},
						},
						updated_at: chrono::Utc::now(),
						search_updated_at: None,
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				let change = ChangeField {
					key: "name".to_string(),
					ty: ChangeFieldType::String,
					old_value: self.emote.default_name.clone().into(),
					value: name.into(),
					..Default::default()
				};
				changes.push(change.clone());
				nested_changes.push(change);
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

					if self.emote.flags.contains(EmoteFlags::PublicListed) != listed {
						let change = ChangeField {
							key: "listed".to_string(),
							ty: ChangeFieldType::Bool,
							old_value: self.emote.flags.contains(EmoteFlags::PublicListed).into(),
							value: listed.into(),
							..Default::default()
						};
						changes.push(change.clone());
						nested_changes.push(change);
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

					AuditLog::collection(&global.db)
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
							updated_at: chrono::Utc::now(),
							search_updated_at: None,
						})
						.session(&mut session)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to insert audit log");
							ApiError::INTERNAL_SERVER_ERROR
						})?;

					changes.push(ChangeField {
						key: "owner_id".to_string(),
						ty: ChangeFieldType::String,
						old_value: self.emote.owner_id.to_string().into(),
						value: owner_id.0.to_string().into(),
						..Default::default()
					});
				}
			}

			if flags != self.emote.flags {
				update.insert("flags", flags.bits());

				AuditLog::collection(&global.db)
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
						updated_at: chrono::Utc::now(),
						search_updated_at: None,
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				changes.push(ChangeField {
					key: "flags".to_string(),
					ty: ChangeFieldType::Number,
					old_value: self.emote.flags.bits().into(),
					value: flags.bits().into(),
					..Default::default()
				});
			}

			if let Some(tags) = params.tags {
				update.insert("tags", &tags);

				AuditLog::collection(&global.db)
					.insert_one(AuditLog {
						id: AuditLogId::new(),
						actor_id: Some(user.id),
						data: AuditLogData::Emote {
							target_id: self.id.id(),
							data: AuditLogEmoteData::ChangeTags {
								old: self.emote.tags.clone(),
								new: tags.clone(),
							},
						},
						updated_at: Utc::now(),
						search_updated_at: None,
					})
					.session(&mut session)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert audit log");
						ApiError::INTERNAL_SERVER_ERROR
					})?;

				changes.push(ChangeField {
					key: "tags".to_string(),
					old_value: self.emote.tags.clone().into(),
					value: tags.into(),
					..Default::default()
				});
			}

			update.insert("updated_at", Some(bson::DateTime::from(chrono::Utc::now())));

			let emote = shared::database::emote::Emote::collection(&global.db)
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

			if !nested_changes.is_empty() {
				let nested_changes = serde_json::to_value(nested_changes).map_err(|e| {
					tracing::error!(error = %e, "failed to serialize nested changes");
					ApiError::INTERNAL_SERVER_ERROR
				})?;

				changes.push(ChangeField {
					key: "versions".to_string(),
					nested: true,
					index: Some(0),
					value: nested_changes,
					..Default::default()
				});
			}

			if !changes.is_empty() {
				let body = ChangeMap {
					id: self.id.id(),
					kind: shared::event_api::types::ObjectKind::Emote,
					actor: Some(UserPartialModel::from_db(
						user.clone(),
						None,
						None,
						&global.config.api.cdn_origin,
					)),
					updated: changes,
					..Default::default()
				};

				global
					.event_api
					.dispatch_event(EventType::UpdateEmote, body, self.id.0)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to dispatch event");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}

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

		let mut session = global.mongo.start_session().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|e| {
			tracing::error!(error = %e, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let emote = DbEmote::collection(&global.db)
			.find_one_and_update(
				filter::filter! {
					DbEmote {
						#[filter(rename = "_id")]
						id: self.id.id(),
					}
				},
				update::update! {
					#[update(set)]
					DbEmote {
						merged: EmoteMerged {
							target_id: target_id.id(),
							at: chrono::Utc::now(),
						},
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update emote");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::NOT_FOUND)?;

		AuditLog::collection(&global.db)
			.insert_one(AuditLog {
				id: AuditLogId::new(),
				actor_id: Some(auth_session.user_id()),
				data: AuditLogData::Emote {
					target_id: self.id.id(),
					data: AuditLogEmoteData::Merge {
						new_emote_id: target_id.id(),
					},
				},
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
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
