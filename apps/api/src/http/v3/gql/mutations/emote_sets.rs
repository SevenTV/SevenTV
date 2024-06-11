use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, InputObject, Object, SimpleObject};
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::{self, Collection, EmoteSetPermission, UserEditorState};
use shared::old_types::{EmoteObjectId, EmoteSetObjectId, UserObjectId};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::{ActiveEmote, EmoteSet};

#[derive(Default)]
pub struct EmoteSetsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetsMutation {
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteSetObjectId) -> Result<Option<EmoteSetOps>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_set = global
			.emote_set_by_id_loader()
			.load(id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(emote_set.map(|s| EmoteSetOps {
			id: s.id.into(),
			_emote_set: s,
		}))
	}

	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Create)")]
	async fn create_emote_set<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		user_id: UserObjectId,
		data: CreateEmoteSetInput,
	) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		let user = global
			.user_by_id_loader()
			.load(global, auth_session.user_id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::UNAUTHORIZED)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = {
			let mut roles = global
				.role_by_id_loader()
				.load_many(user.entitled_cache.role_ids.iter().copied())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

			global_config
				.role_ids
				.iter()
				.filter_map(|id| roles.remove(id))
				.collect::<Vec<_>>()
		};

		let permissions = user.compute_permissions(&roles);

		let editors = global
			.user_editor_by_user_id_loader()
			.load(user_id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		if !(auth_session.user_id() == user_id.id()
			|| permissions.has(EmoteSetPermission::Admin)
			|| editors.iter().any(|editor| {
				editor.state == UserEditorState::Accepted
					&& editor.user_id == auth_session.user_id()
					&& editor.permissions.has_emote_set(EmoteSetPermission::Create)
			})) {
			return Err(ApiError::FORBIDDEN);
		}

		let mut flags = database::EmoteSetFlags::default();
		if data.privileged.unwrap_or(false) {
			flags |= database::EmoteSetFlags::Privileged;
		}

		let emote_set = database::EmoteSet {
			owner_id: Some(user_id.id()),
			name: data.name,
			capacity: 300,
			flags,
			..Default::default()
		};

		database::EmoteSet::collection(global.db())
			.insert_one(&emote_set, None)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert emote set");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(EmoteSet::from_db(emote_set))
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateEmoteSetInput {
	name: String,
	privileged: Option<bool>,
}

#[derive(SimpleObject, Default)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct EmoteSetOps {
	id: EmoteSetObjectId,
	#[graphql(skip)]
	_emote_set: database::EmoteSet,
}

impl EmoteSetOps {
	async fn check_perms(&self, global: &Arc<Global>, auth_session: &AuthSession, editor_perm: EmoteSetPermission) -> Result<(), ApiError> {
		let user = global
			.user_by_id_loader()
			.load(global, auth_session.user_id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::UNAUTHORIZED)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = {
			let mut roles = global
				.role_by_id_loader()
				.load_many(user.entitled_cache.role_ids.iter().copied())
				.await
				.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

			global_config
				.role_ids
				.iter()
				.filter_map(|id| roles.remove(id))
				.collect::<Vec<_>>()
		};

		let permissions = user.compute_permissions(&roles);

		if let Some(owner_id) = self._emote_set.owner_id {
			let editors = global
				.user_editor_by_user_id_loader()
				.load(owner_id)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.unwrap_or_default();

			if !(auth_session.user_id() == owner_id
				|| permissions.has(EmoteSetPermission::Admin)
				|| editors.iter().any(|editor| {
					editor.state == UserEditorState::Accepted
						&& editor.user_id == auth_session.user_id()
						&& editor.permissions.has_emote_set(editor_perm)
				})) {
				return Err(ApiError::FORBIDDEN);
			}
		} else {
			if !permissions.has(EmoteSetPermission::Admin) {
				return Err(ApiError::FORBIDDEN);
			}
		}

		Ok(())
	}
}

#[derive(Enum, Copy, Clone, PartialEq, Eq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ListItemAction {
	Add,
	Update,
	Remove,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UpdateEmoteSetInput {
	name: Option<String>,
	capacity: Option<u32>,
	origins: Option<Vec<EmoteSetOriginInput>>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EmoteSetOriginInput {
	id: EmoteSetObjectId,
	weight: Option<u32>,
	slices: Option<Vec<u32>>,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl EmoteSetOps {
	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Edit)")]
	async fn emotes<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		id: EmoteObjectId,
		action: ListItemAction,
		name: Option<String>,
	) -> Result<Vec<ActiveEmote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		self.check_perms(global, auth_session, EmoteSetPermission::Edit).await?;

		let mut session = global.mongo().start_session(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction(None).await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		match action {
			ListItemAction::Add => {
				let emote = global
					.emote_by_id_loader()
					.load(id.id())
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::NOT_FOUND)?;

				let emote_set_emote = database::EmoteSetEmote {
					emote_set_id: self._emote_set.id,
					emote_id: id.id(),
					added_by_id: Some(auth_session.user_id()),
					name: name.unwrap_or(emote.default_name),
					..Default::default()
				};

				database::EmoteSetEmote::collection(global.db())
					.insert_one(&emote_set_emote, None)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to insert emote set emote");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}
			ListItemAction::Remove => {
				database::EmoteSetEmote::collection(global.db())
					.delete_one(
						doc! {
							"emote_set_id": self._emote_set.id,
							"emote_id": id.id(),
						},
						None,
					)
					.await
					.map_err(|e| {
						tracing::error!(error = %e, "failed to delete emote set emote");
						ApiError::INTERNAL_SERVER_ERROR
					})?;
			}
			ListItemAction::Update => {
				if let Some(name) = name {
					database::EmoteSetEmote::collection(global.db())
						.update_one(
							doc! {
								"emote_set_id": self._emote_set.id,
								"emote_id": id.id(),
							},
							doc! {
								"$set": {
									"name": name,
								},
							},
							None,
						)
						.await
						.map_err(|e| {
							tracing::error!(error = %e, "failed to update emote set emote");
							ApiError::INTERNAL_SERVER_ERROR
						})?;
				}
			}
		}

		let active_emotes = global
			.emote_set_emote_by_id_loader()
			.load(self._emote_set.id)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(active_emotes.into_iter().map(|e| ActiveEmote::from_db(e)).collect())
	}

	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Edit)")]
	async fn update<'ctx>(&self, ctx: &Context<'ctx>, data: UpdateEmoteSetInput) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		self.check_perms(global, auth_session, EmoteSetPermission::Edit).await?;

		let mut update = doc! {};

		if let Some(name) = data.name {
			update.insert("name", name);
		}

		let emote_set = database::EmoteSet::collection(global.db())
			.find_one_and_update(
				doc! { "_id": self._emote_set.id },
				doc! { "$set": update },
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.build(),
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update emote set");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(EmoteSet::from_db(emote_set))
	}

	#[graphql(guard = "PermissionGuard::one(EmoteSetPermission::Delete)")]
	async fn delete<'ctx>(&self, ctx: &Context<'ctx>) -> Result<bool, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		self.check_perms(global, auth_session, EmoteSetPermission::Delete).await?;

		let res = database::EmoteSet::collection(global.db())
			.delete_one(doc! { "_id": self._emote_set.id }, None)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to delete emote set");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(res.deleted_count == 1)
	}
}
