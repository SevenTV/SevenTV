use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object};
use shared::database::UserId;
use shared::old_types::{CosmeticBadgeModel, CosmeticKind, CosmeticPaintModel, UserConnectionPlatformModel, UserTypeModel};

use super::audit_logs::AuditLog;
use super::emote_sets::EmoteSet;
use super::emotes::Emote;
use super::reports::Report;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::object_id::{
	BadgeObjectId, EmoteSetObjectId, ObjectId, PaintObjectId, RoleObjectId, UserObjectId,
};
use crate::http::v3::types::UserEditorModelPermission;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/users.gql

#[derive(Default)]
pub struct UsersQuery;

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct User {
	id: UserObjectId,
	#[graphql(name = "type")]
	user_type: UserTypeModel,
	username: String,
	display_name: String,
	// created_at
	avatar_url: String,
	biography: String,
	style: UserStyle,

	// editors
	// editor_of
	// cosmetics
	roles: Vec<RoleObjectId>,

	// emote_sets
	// owned_emotes
	// activity
	// connections
	inbox_unread_count: u32,
	// reports
	#[graphql(skip)]
	db_permissions: shared::database::Permissions,
	#[graphql(skip)]
	db_connections: Vec<shared::database::UserConnection>,
}

impl From<UserPartial> for User {
	fn from(partial: UserPartial) -> Self {
		Self {
			id: partial.id,
			user_type: partial.user_type,
			username: partial.username,
			display_name: partial.display_name,
			avatar_url: partial.avatar_url,
			biography: partial.biography,
			style: partial.style,
			roles: partial.roles,
			inbox_unread_count: 0,
			db_permissions: partial.db_permissions,
			db_connections: partial.db_connections,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl User {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.id().timestamp()
	}

	async fn editors<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let editors = global
			.user_editor_by_user_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, false)).collect())
	}

	async fn editors_of<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let editors = global
			.user_editor_by_editor_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, true)).collect())
	}

	async fn cosmetics(&self) -> Result<Vec<UserCosmetic>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn emote_sets<'ctx>(&self, ctx: &Context<'ctx>, _entitled: Option<bool>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_sets = global
			.emote_set_by_user_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		// TODO: query entitleled sets too

		Ok(emote_sets.into_iter().map(|e| EmoteSet::from_db(e)).collect())
	}

	async fn owned_emotes<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Emote>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn activity(&self) -> Result<Vec<AuditLog>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn connections(&self) -> Vec<UserConnection> {
		self.db_connections
			.iter()
			.map(|c| UserConnection::from_db(c.clone(), self.db_permissions.emote_set_slots_limit.unwrap_or(600)))
			.collect()
	}

	async fn reports(&self) -> Vec<Report> {
		// always empty because user reports were never implemented
		vec![]
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserEditor {
	id: UserObjectId,
	// user
	permissions: UserEditorModelPermission,
	visible: bool,
	added_at: chrono::DateTime<chrono::Utc>,
}

impl UserEditor {
	fn from_db(value: shared::database::UserEditor, editor_of: bool) -> Option<Self> {
		if value.state != shared::database::UserEditorState::Accepted {
			return None;
		}

		Some(UserEditor {
			id: editor_of.then_some(value.user_id.into()).unwrap_or(value.editor_id.into()),
			added_at: value.id.timestamp(),
			permissions: UserEditorModelPermission::ModifyEmotes | UserEditorModelPermission::ManageEmoteSets,
			visible: true,
		})
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserEditor {
	async fn user<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(UserPartial::load_from_db(global, self.id.id()).await?)
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserCosmetic {
	id: ObjectId<()>,
	selected: bool,
	kind: CosmeticKind,
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserPartial {
	id: UserObjectId,
	#[graphql(name = "type")]
	user_type: UserTypeModel,
	username: String,
	display_name: String,
	// created_at
	avatar_url: String,
	biography: String,
	style: UserStyle,
	roles: Vec<RoleObjectId>,
	// connections
	// emote_sets
	#[graphql(skip)]
	db_permissions: shared::database::Permissions,
	#[graphql(skip)]
	db_connections: Vec<shared::database::UserConnection>,
}

impl UserPartial {
	pub async fn load_from_db(global: &Arc<Global>, id: UserId) -> Result<Self, ApiError> {
		Self::load_many_from_db(global, [id])
			.await?
			.into_iter()
			.next()
			.ok_or(ApiError::NOT_FOUND)
	}

	pub async fn load_many_from_db(
		global: &Arc<Global>,
		ids: impl IntoIterator<Item = UserId> + Clone,
	) -> Result<Vec<Self>, ApiError> {
		let ids: Vec<_> = ids.into_iter().collect();

		let users = global
			.user_by_id_loader()
			.load_many(global, ids.clone())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut all_connections = global
			.user_connection_by_user_id_loader()
			.load_many(ids)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = {
			let role_ids: Vec<_> = users
				.iter()
				.map(|(_, u)| u.entitled_cache.role_ids.iter().copied())
				.flatten()
				.collect();

			let mut roles = global
				.role_by_id_loader()
				.load_many(role_ids)
				.await
				.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

			global_config
				.role_ids
				.iter()
				.filter_map(|id| roles.remove(id))
				.collect::<Vec<_>>()
		};

		let mut result = Vec::new();

		for (id, user) in users {
			let permissions = user.compute_permissions(&roles);
			let connections = all_connections.remove(&id).unwrap_or_default();
			result.push(UserPartial::from_db(global, user, permissions, connections));
		}

		Ok(result)
	}

	pub fn from_db(
		global: &Arc<Global>,
		user: shared::database::User,
		permissions: shared::database::Permissions,
		connections: Vec<shared::database::UserConnection>,
	) -> Self {
		let main_connection = connections.iter().find(|c| c.main_connection);

		let avatar_url = user
			.style
			.active_profile_picture
			.and_then(|s| {
				s.outputs
					.iter()
					.max_by_key(|i| i.size)
					.map(|i| i.get_url(&global.config().api.cdn_base_url))
			})
			.or(main_connection.and_then(|c| c.platform_avatar_url.clone()));

		Self {
			id: user.id.into(),
			user_type: UserTypeModel::Regular,
			username: main_connection.map(|c| c.platform_username.clone()).unwrap_or_default(),
			display_name: main_connection.map(|c| c.platform_display_name.clone()).unwrap_or_default(),
			avatar_url: avatar_url.unwrap_or_default(),
			biography: String::new(),
			style: UserStyle {
				color: 0,
				paint_id: user.style.active_paint_id.map(Into::into),
				badge_id: user.style.active_badge_id.map(Into::into),
			},
			roles: user.grants.role_ids.into_iter().map(Into::into).collect(),
			db_permissions: permissions,
			db_connections: connections,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserPartial {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.id().timestamp()
	}

	async fn connections(&self) -> Vec<UserConnection> {
		self.db_connections
			.iter()
			.map(|c| UserConnection::from_db(c.clone(), self.db_permissions.emote_set_slots_limit.unwrap_or(600)))
			.collect()
	}

	async fn emote_sets<'ctx>(&self, ctx: &Context<'ctx>, _entitled: Option<bool>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_sets = global
			.emote_set_by_user_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(emote_sets.into_iter().map(|e| EmoteSet::from_db(e)).collect())
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserConnection {
	id: String,
	platform: UserConnectionPlatformModel,
	username: String,
	display_name: String,
	linked_at: chrono::DateTime<chrono::Utc>,
	emote_capacity: i32,
	emote_set_id: Option<EmoteSetObjectId>,
}

impl UserConnection {
	fn from_db(value: shared::database::UserConnection, slots: u16) -> Self {
		Self {
			id: value.platform_id,
			platform: value.platform.into(),
			username: value.platform_username,
			display_name: value.platform_display_name,
			linked_at: value.id.timestamp(),
			emote_capacity: slots as i32,
			emote_set_id: Some(value.user_id.cast().into()),
		}
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserStyle {
	color: i32,
	paint_id: Option<PaintObjectId>,
	// paint
	badge_id: Option<BadgeObjectId>,
	// badge
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserStyle {
	async fn paint<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<CosmeticPaintModel>, ApiError> {
		let Some(id) = self.paint_id else {
			return Ok(None);
		};

		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.paint_by_id_loader()
			.load(id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.and_then(|p| CosmeticPaintModel::from_db(p, &global.config().api.cdn_base_url)))
	}

	async fn badge<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<CosmeticBadgeModel>, ApiError> {
		let Some(id) = self.badge_id else {
			return Ok(None);
		};

		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.badge_by_id_loader()
			.load(id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.and_then(|b| CosmeticBadgeModel::from_db(b, &global.config().api.cdn_base_url)))
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
pub struct UserSearchResult {
	total: u32,
	items: Vec<UserPartial>,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl UsersQuery {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<User>, ApiError> {
		let Some(session) = ctx.data_opt::<AuthSession>() else {
			return Ok(None);
		};
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let id = session.user_id();
		Ok(Some(UserPartial::load_from_db(global, id).await?.into()))
	}

	async fn user<'ctx>(&self, ctx: &Context<'ctx>, id: UserObjectId) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(UserPartial::load_from_db(global, id.id()).await?.into())
	}

	async fn user_by_connection<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		platform: UserConnectionPlatformModel,
		id: String,
	) -> Result<User, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn users<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		query: String,
		page: Option<u32>,
		limit: Option<u32>,
	) -> Result<UserPartial, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(name = "usersByID")]
	async fn users_by_id<'ctx>(&self, ctx: &Context<'ctx>, list: Vec<UserObjectId>) -> Result<Vec<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		UserPartial::load_many_from_db(global, list.into_iter().map(|id| id.id())).await
	}
}
