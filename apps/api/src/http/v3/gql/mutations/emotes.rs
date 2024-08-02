use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::emote::{Emote as DbEmote, EmoteFlags, EmoteMerged};
use shared::database::event::{Event, EventData, EventEmoteData, EventId};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{EmotePermission, PermissionsExt};
use shared::database::user::editor::{EditorEmotePermission, UserEditorId, UserEditorState};
use shared::event_api::types::{ChangeField, ChangeFieldType, ChangeMap, EventType};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{EmoteFlagsModel, UserPartialModel};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::emote::Emote;
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

				tx.insert_one(
					Event {
						id: EventId::new(),
						actor_id: Some(user.id),
						data: EventData::Emote {
							target_id: emote.id,
							data: EventEmoteData::Delete,
						},
						updated_at: chrono::Utc::now(),
						search_updated_at: None,
					},
					None,
				)
				.await?;

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
				let mut changes = vec![];
				let mut nested_changes = vec![];

				let new_default_name = if let Some(name) = params.name.or(params.version_name) {
					tx.insert_one(
						Event {
							id: EventId::new(),
							actor_id: Some(user.id),
							data: EventData::Emote {
								target_id: self.id.id(),
								data: EventEmoteData::ChangeName {
									old: self.emote.default_name.clone(),
									new: name.clone(),
								},
							},
							updated_at: chrono::Utc::now(),
							search_updated_at: None,
						},
						None,
					)
					.await?;

					let change = ChangeField {
						key: "name".to_string(),
						ty: ChangeFieldType::String,
						old_value: self.emote.default_name.clone().into(),
						value: name.clone().into(),
						..Default::default()
					};
					changes.push(change.clone());
					nested_changes.push(change);

					Some(name)
				} else {
					None
				};

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
						tx.insert_one(
							Event {
								id: EventId::new(),
								actor_id: Some(user.id),
								data: EventData::Emote {
									target_id: self.id.id(),
									data: EventEmoteData::ChangeOwner {
										old: self.emote.owner_id,
										new: owner_id.id(),
									},
								},
								updated_at: chrono::Utc::now(),
								search_updated_at: None,
							},
							None,
						)
						.await?;

						changes.push(ChangeField {
							key: "owner_id".to_string(),
							ty: ChangeFieldType::String,
							old_value: self.emote.owner_id.to_string().into(),
							value: owner_id.0.to_string().into(),
							..Default::default()
						});

						Some(owner_id.id())
					} else {
						None
					}
				} else {
					None
				};

				let new_flags = if flags != self.emote.flags {
					tx.insert_one(
						Event {
							id: EventId::new(),
							actor_id: Some(user.id),
							data: EventData::Emote {
								target_id: self.id.id(),
								data: EventEmoteData::ChangeFlags {
									old: self.emote.flags,
									new: flags,
								},
							},
							updated_at: chrono::Utc::now(),
							search_updated_at: None,
						},
						None,
					)
					.await?;

					changes.push(ChangeField {
						key: "flags".to_string(),
						ty: ChangeFieldType::Number,
						old_value: self.emote.flags.bits().into(),
						value: flags.bits().into(),
						..Default::default()
					});

					Some(flags)
				} else {
					None
				};

				if let Some(tags) = &params.tags {
					tx.insert_one(
						Event {
							id: EventId::new(),
							actor_id: Some(user.id),
							data: EventData::Emote {
								target_id: self.id.id(),
								data: EventEmoteData::ChangeTags {
									old: self.emote.tags.clone(),
									new: tags.clone(),
								},
							},
							updated_at: Utc::now(),
							search_updated_at: None,
						},
						None,
					)
					.await?;

					changes.push(ChangeField {
						key: "tags".to_string(),
						old_value: self.emote.tags.clone().into(),
						value: tags.clone().into(),
						..Default::default()
					});
				}

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
								default_name: new_default_name,
								#[query(optional)]
								owner_id: new_owner_id,
								#[query(optional)]
								flags: new_flags,
								#[query(optional)]
								tags: params.tags,
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

				if !nested_changes.is_empty() {
					let nested_changes = serde_json::to_value(nested_changes)
						.map_err(|e| {
							tracing::error!(error = %e, "failed to serialize nested changes");
							ApiError::INTERNAL_SERVER_ERROR
						})
						.map_err(TransactionError::custom)?;

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
						})
						.map_err(TransactionError::custom)?;
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

			tx.insert_one(
				Event {
					id: EventId::new(),
					actor_id: Some(auth_session.user_id()),
					data: EventData::Emote {
						target_id: self.id.id(),
						data: EventEmoteData::Merge {
							new_emote_id: target_id.id(),
						},
					},
					updated_at: chrono::Utc::now(),
					search_updated_at: None,
				},
				None,
			)
			.await?;

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
