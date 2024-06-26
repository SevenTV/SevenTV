use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object};
use data_source::UserDataSource;
use mongodb::bson::doc;
use shared::database::activity::EmoteSetActivityData;
use shared::database::user::UserId;
use shared::old_types::cosmetic::{CosmeticBadgeModel, CosmeticKind, CosmeticPaintModel};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{UserConnectionPlatformModel, UserTypeModel};
use tokio::sync::RwLock;

use super::audit_logs::AuditLog;
use super::emote_sets::EmoteSet;
use super::emotes::Emote;
use super::reports::Report;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::types::UserEditorModelPermission;

mod data_source;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/users.gql

#[derive(Default)]
pub struct UsersQuery;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct User {
	id: GqlObjectId,
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
	// roles,

	// emote_sets
	// owned_emotes
	// activity
	// connections
	inbox_unread_count: u32,
	// reports
	#[graphql(skip)]
	db: Arc<RwLock<UserDataSource>>,
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
			inbox_unread_count: 0,
			db: partial.db,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl User {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	async fn editors<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let editors = global
			.user_editor_by_user_id_loader()
			.load(self.id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, false)).collect())
	}

	async fn editor_of<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let editors = global
			.user_editor_by_editor_id_loader()
			.load(self.id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, true)).collect())
	}

	async fn cosmetics<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserCosmetic>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut guard = self.db.write().await;
		let full = guard.full(global).await?;

		let paints = full
			.computed
			.entitlements
			.paints
			.iter()
			.map(|p| (CosmeticKind::Paint, p.cast::<()>()));
		let badges = full
			.computed
			.entitlements
			.badges
			.iter()
			.map(|b| (CosmeticKind::Badge, b.cast::<()>()));

		let cosmetics = paints
			.chain(badges)
			.map(|(kind, id)| UserCosmetic {
				id: id.into(),
				selected: full.user.style.active_paint_id.is_some_and(|p| p == id.cast())
					|| full.user.style.active_badge_id.is_some_and(|b| b == id.cast()),
				kind,
			})
			.collect();

		Ok(cosmetics)
	}

	async fn roles<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<GqlObjectId>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut guard = self.db.write().await;

		Ok(guard
			.full(global)
			.await?
			.computed
			.entitlements
			.roles
			.iter()
			.map(|r| (*r).into())
			.collect())
	}

	async fn emote_sets<'ctx>(&self, ctx: &Context<'ctx>, _entitled: Option<bool>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_sets = global
			.emote_set_by_user_id_loader()
			.load(self.id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(emote_sets.into_iter().map(|e| EmoteSet::from_db(e)).collect())
	}

	async fn owned_emotes<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Emote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emotes = global
			.emote_by_user_id_loader()
			.load(self.id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(emotes.into_iter().map(|e| Emote::from_db(global, e)).collect())
	}

	async fn activity<'ctx>(&self, ctx: &Context<'ctx>, limit: Option<u32>) -> Result<Vec<AuditLog>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let activities = global
			.emote_set_activity_by_actor_id_loader()
			.load(self.id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		let mut emote_ids = vec![];

		for a in &activities {
			if let Some(EmoteSetActivityData::ChangeEmotes { added, removed }) = &a.data {
				emote_ids.extend(added);
				emote_ids.extend(removed);
			}
		}

		let emotes = global
			.emote_by_id_loader()
			.load_many(emote_ids)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(activities
			.into_iter()
			.map(|a| AuditLog::from_db_emote_set(a, &emotes))
			.collect())
	}

	async fn connections(&self) -> Vec<UserConnection> {
		self.db
			.read()
			.await
			.user()
			.connections
			.iter()
			.map(|c| UserConnection {
				id: c.platform_id.clone(),
				platform: c.platform.into(),
				username: c.platform_username.clone(),
				display_name: c.platform_display_name.clone(),
				linked_at: c.linked_at,
				emote_capacity: todo!(),
				emote_set_id: todo!(),
			})
			.collect()
	}

	async fn reports(&self) -> Vec<Report> {
		// always empty because user reports were never implemented
		vec![]
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserEditor {
	id: GqlObjectId,
	// user
	permissions: UserEditorModelPermission,
	visible: bool,
	added_at: chrono::DateTime<chrono::Utc>,
}

impl UserEditor {
	pub fn from_db(value: shared::database::user::editor::UserEditor, editor_of: bool) -> Option<Self> {
		if value.state != shared::database::user::editor::UserEditorState::Accepted {
			return None;
		}

		Some(UserEditor {
			id: editor_of.then_some(value.user_id.into()).unwrap_or(value.editor_id.into()),
			added_at: value.added_at,
			permissions: UserEditorModelPermission::from_db(&value.permissions),
			visible: true,
		})
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserEditor {
	async fn user<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.user_by_id_loader()
			.load(self.id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u.into()))
			.unwrap_or_else(|| UserPartial::deleted_user()))
	}
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UserCosmetic {
	id: GqlObjectId,
	selected: bool,
	kind: CosmeticKind,
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserPartial {
	id: GqlObjectId,
	#[graphql(name = "type")]
	user_type: UserTypeModel,
	username: String,
	display_name: String,
	// created_at
	avatar_url: String,
	biography: String,
	style: UserStyle,
	// roles
	// connections
	// emote_sets
	#[graphql(skip)]
	db: Arc<RwLock<UserDataSource>>,
}

impl UserPartial {
	pub fn deleted_user() -> Self {
		Self {
			id: GqlObjectId::from(UserId::nil()),
			user_type: UserTypeModel::Regular,
			username: "*DeletedUser".to_string(),
			display_name: "*DeletedUser".to_string(),
			avatar_url: String::new(),
			biography: String::new(),
			style: UserStyle {
				color: 0,
				paint_id: None,
				badge_id: None,
			},
			db: Arc::new(RwLock::new(shared::database::user::User::default().into())),
		}
	}

	pub fn from_db(global: &Arc<Global>, source: UserDataSource) -> Self {
		let user = source.user();

		let main_connection = user.connections.first();

		let avatar_url = user
			.style
			.active_profile_picture
			.as_ref()
			.and_then(|s| {
				s.outputs
					.iter()
					.max_by_key(|i| i.size)
					.map(|i| i.get_url(&global.config().api.cdn_origin))
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
				paint_id: user.style.active_paint_id.clone().map(Into::into),
				badge_id: user.style.active_badge_id.clone().map(Into::into),
			},
			db: Arc::new(RwLock::new(source)),
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserPartial {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	async fn roles<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<GqlObjectId>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut guard = self.db.write().await;

		Ok(guard
			.full(global)
			.await?
			.computed
			.entitlements
			.roles
			.iter()
			.map(|r| (*r).into())
			.collect())
	}

	async fn connections(&self) -> Vec<UserConnection> {
		self.db
			.read()
			.await
			.user()
			.connections
			.iter()
			.map(|c| UserConnection {
				id: c.platform_id.clone(),
				platform: c.platform.into(),
				username: c.platform_username.clone(),
				display_name: c.platform_display_name.clone(),
				linked_at: c.linked_at,
				emote_capacity: todo!(),
				emote_set_id: todo!(),
			})
			.collect()
	}

	async fn emote_sets<'ctx>(&self, ctx: &Context<'ctx>, _entitled: Option<bool>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_sets = global
			.emote_set_by_user_id_loader()
			.load(self.id.0.cast())
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
	emote_set_id: Option<GqlObjectId>,
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct UserStyle {
	color: i32,
	// paint
	paint_id: Option<GqlObjectId>,
	// badge
	badge_id: Option<GqlObjectId>,
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
			.load(id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.and_then(|p| CosmeticPaintModel::from_db(p, &global.config().api.cdn_origin)))
	}

	async fn badge<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<CosmeticBadgeModel>, ApiError> {
		let Some(id) = self.badge_id else {
			return Ok(None);
		};

		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.badge_by_id_loader()
			.load(id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.and_then(|b| CosmeticBadgeModel::from_db(b, &global.config().api.cdn_origin)))
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

		let user = global
			.user_loader()
			.load(global, session.user_id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u.into()));

		Ok(user.map(Into::into))
	}

	async fn user<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let user = global
			.user_loader()
			.load(global, id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u.into()))
			.unwrap_or_else(|| UserPartial::deleted_user());

		Ok(user.into())
	}

	async fn user_by_connection<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		platform: UserConnectionPlatformModel,
		id: String,
	) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let platform = shared::database::user::connection::Platform::from(platform);

		let user = global
			.user_by_platform_id_loader()
			.load((platform, id))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u.into()))
			.unwrap_or_else(|| UserPartial::deleted_user());

		Ok(user.into())
	}

	async fn users<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		query: String,
		page: Option<u32>,
		limit: Option<u32>,
	) -> Result<UserPartial, ApiError> {
		// TODO: implement with typesense
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(name = "usersByID")]
	async fn users_by_id<'ctx>(&self, ctx: &Context<'ctx>, list: Vec<GqlObjectId>) -> Result<Vec<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let users = global
			.user_loader()
			.load_many(global, list.into_iter().map(|id| id.0.cast()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.map(|u| UserPartial::from_db(global, u.into()))
			.collect();

		Ok(users)
	}
}
