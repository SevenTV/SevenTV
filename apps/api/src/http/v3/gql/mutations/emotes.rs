use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::bson::doc;
use shared::database::role::permissions::{EmotePermission, PermissionsExt};
use shared::database::user::editor::{EditorEmotePermission, UserEditorState};
use shared::database::Collection;
use shared::database::emote::Emote as DbEmote;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::EmoteFlagsModel;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::Emote;

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
	#[graphql(guard = "PermissionGuard::one(EmotePermission::Edit)")]
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

		// TODO(troy): resume work from here

		if params.deleted.is_some_and(|d| d) {
			if !perms.has(EmotePermission::Delete) {
				return Err(ApiError::FORBIDDEN);
			}

			let emote = database::Emote::collection(global.db())
				.find_one_and_delete(doc! { "_id": self.id.id() }, None)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				.ok_or(ApiError::NOT_FOUND)?;

			Ok(Emote::from_db(global, emote))
		} else {
			if !perms.has(EmotePermission::Edit) {
				return Err(ApiError::FORBIDDEN);
			}

			let mut update = doc! {};

			if let Some(name) = params.name.or(params.version_name) {
				update.insert("default_name", name);
			}

			if let Some(tags) = params.tags {
				update.insert("tags", tags);
			}

			let mut flags = self.emote.flags;

			if let Some(input_flags) = params.flags {
				if input_flags.contains(EmoteFlagsModel::Private) {
					flags |= database::EmoteFlags::Private;
					flags &= !database::EmoteFlags::PublicListed;
				} else {
					flags &= !database::EmoteFlags::Private;
					flags |= database::EmoteFlags::PublicListed;
				}

				if input_flags.contains(EmoteFlagsModel::ZeroWidth) {
					flags |= database::EmoteFlags::DefaultZeroWidth;
				} else {
					flags &= !database::EmoteFlags::DefaultZeroWidth;
				}
			}

			// changing visibility and owner requires admin perms
			if perms.has(EmotePermission::Admin) {
				if let Some(listed) = params.listed {
					if listed {
						flags |= database::EmoteFlags::PublicListed;
						flags &= !database::EmoteFlags::Private;
					} else {
						flags &= !database::EmoteFlags::PublicListed;
						flags |= database::EmoteFlags::Private;
					}
				}

				if let Some(personal_use) = params.personal_use {
					if personal_use {
						flags |= database::EmoteFlags::ApprovedPersonal;
						flags &= !database::EmoteFlags::DeniedPersonal;
					} else {
						flags &= !database::EmoteFlags::ApprovedPersonal;
						flags |= database::EmoteFlags::DeniedPersonal;
					}
				}

				if let Some(owner_id) = params.owner_id {
					update.insert("owner_id", owner_id.id());
				}
			}

			update.insert("flags", flags.bits() as u32);

			let emote = database::Emote::collection(global.db())
				.find_one_and_update(doc! { "_id": self.id.id() }, doc! { "$set": update }, None)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to update emote");
					ApiError::INTERNAL_SERVER_ERROR
				})?
				.ok_or(ApiError::NOT_FOUND)?;

			Ok(Emote::from_db(global, emote))
		}
	}

	#[graphql(guard = "PermissionGuard::one(EmotePermission::Admin)")]
	async fn merge<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		target_id: GqlObjectId,
		_reason: Option<String>,
	) -> Result<Emote, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote = database::Emote::collection(global.db())
			.find_one_and_update(
				doc! { "_id": self.id.id() },
				doc! {
					"$set": {
						"merged_into": target_id.id(),
					},
					"$currentDate": {
						"merged_at": { "$type": "date" },
					},
				},
				None,
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update emote");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::NOT_FOUND)?;

		// TODO: schedule emote merge job

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
