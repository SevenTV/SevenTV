use std::future::IntoFuture;
use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object};
use futures::{TryFutureExt, TryStreamExt};
use mongodb::bson::doc;
use shared::database::audit_log::{AuditLogData, AuditLogEmoteSetData};
use shared::database::global::GlobalConfig;
use shared::database::user::UserId;
use shared::database::Collection;
use shared::old_types::cosmetic::{CosmeticBadgeModel, CosmeticKind, CosmeticPaintModel};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{UserConnectionPlatformModel, UserTypeModel};
use tokio::sync::RwLock;

use self::data_source::UserDataSource;
use super::audit_log::AuditLog;
use super::emote::Emote;
use super::emote_set::EmoteSet;
use super::report::Report;
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
	// style

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

	async fn style<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserStyle, ApiError> {
		let global = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut guard = self.db.write().await;

		let Some(full) = guard.full(global).await.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)? else {
			return Ok(UserStyle::default());
		};

		Ok(UserStyle {
			color: full.computed.highest_role_color.unwrap_or_default(),
			paint_id: full.user.style.active_paint_id.map(Into::into),
			badge_id: full.user.style.active_badge_id.map(Into::into),
		})
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

	async fn editor_of<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let editors = global
			.user_editor_by_editor_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, true)).collect())
	}

	async fn cosmetics<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserCosmetic>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut guard = self.db.write().await;
		let Some(full) = guard.full(global).await? else {
			return Ok(vec![]);
		};

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

		let Some(full) = guard.full(global).await? else {
			return Ok(vec![]);
		};

		let mut roles: Vec<_> = global
			.role_by_id_loader()
			.load_many(full.computed.entitlements.roles.iter().copied())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.collect();
		roles.sort_by_key(|r| r.rank);
		roles.reverse();

		Ok(roles.into_iter().map(|r| r.id.into()).collect())
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

	async fn owned_emotes<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Emote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emotes = global
			.emote_by_user_id_loader()
			.load(self.id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(emotes.into_iter().map(|e| Emote::from_db(global, e)).collect())
	}

	async fn activity<'ctx>(&self, ctx: &Context<'ctx>, limit: Option<u32>) -> Result<Vec<AuditLog>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let Some(emote_set_id) = self.db.read().await.user().style.active_emote_set_id else {
			return Ok(vec![]);
		};

		let audit_logs: Vec<_> = shared::database::audit_log::AuditLog::collection(global.db())
			.find(doc! {
				"$or": [
					{ "data.kind": "user", "data.data.kind": "add_editor", "data.target_id": self.id.0 },
					{ "data.kind": "user", "data.data.kind": "remove_editor", "data.target_id": self.id.0 },
					{ "data.kind": "emote_set", "data.target_id": emote_set_id },
				]
			})
			.sort(doc! { "_id": -1 })
			.limit(limit.unwrap_or(100) as i64)
			.into_future()
			.and_then(|f| f.try_collect())
			.await
			.map_err(|e| {
				tracing::error!(%e, "failed to query audit logs");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let mut emote_ids = vec![];

		for a in &audit_logs {
			match a.data {
				AuditLogData::EmoteSet {
					data: AuditLogEmoteSetData::AddEmote { emote_id, .. },
					..
				}
				| AuditLogData::EmoteSet {
					data: AuditLogEmoteSetData::RemoveEmote { emote_id },
					..
				} => {
					emote_ids.push(emote_id);
				}
				_ => {}
			}
		}

		let emotes = global
			.emote_by_id_loader()
			.load_many(emote_ids)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(audit_logs.into_iter().filter_map(|l| AuditLog::from_db(l, &emotes)).collect())
	}

	async fn connections<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserConnection>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let guard = self.db.read().await;
		let user = guard.user();

		Ok(user
			.connections
			.iter()
			.map(|c| UserConnection::from_db(c.clone(), &user.style, &global_config))
			.collect())
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
			id: editor_of
				.then_some(value.id.user_id.into())
				.unwrap_or(value.id.editor_id.into()),
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
			.load(self.id.id())
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
	// style
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
			db: Arc::new(RwLock::new(source)),
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserPartial {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	async fn style<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserStyle, ApiError> {
		let global = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut guard = self.db.write().await;

		let Some(full) = guard.full(global).await.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)? else {
			return Ok(UserStyle::default());
		};

		Ok(UserStyle {
			color: full.computed.highest_role_color.unwrap_or_default(),
			paint_id: full.user.style.active_paint_id.map(Into::into),
			badge_id: full.user.style.active_badge_id.map(Into::into),
		})
	}

	async fn roles<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<GqlObjectId>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut guard = self.db.write().await;

		let Some(full) = guard.full(global).await? else {
			return Ok(vec![]);
		};

		let mut roles: Vec<_> = global
			.role_by_id_loader()
			.load_many(full.computed.entitlements.roles.iter().copied())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.collect();

		roles.sort_by_key(|r| r.rank);
		roles.reverse();

		Ok(roles.into_iter().map(|r| r.id.into()).collect())
	}

	async fn connections<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserConnection>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let guard = self.db.read().await;
		let user = guard.user();

		Ok(user
			.connections
			.iter()
			.map(|c| UserConnection::from_db(c.clone(), &user.style, &global_config))
			.collect())
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
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

impl UserConnection {
	pub fn from_db(
		connection: shared::database::user::connection::UserConnection,
		style: &shared::database::user::UserStyle,
		global_config: &GlobalConfig,
	) -> Self {
		Self {
			id: connection.platform_id,
			platform: connection.platform.into(),
			username: connection.platform_username,
			display_name: connection.platform_display_name,
			linked_at: connection.linked_at,
			emote_capacity: global_config.normal_emote_set_slot_capacity,
			emote_set_id: style.active_emote_set_id.map(Into::into),
		}
	}
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
			.load(id.id())
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
			.load(id.id())
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
			.load(global, id.id())
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
		_ctx: &Context<'ctx>,
		_query: String,
		_page: Option<u32>,
		_limit: Option<u32>,
	) -> Result<UserPartial, ApiError> {
		// TODO: implement with typesense
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(name = "usersByID")]
	async fn users_by_id<'ctx>(&self, ctx: &Context<'ctx>, list: Vec<GqlObjectId>) -> Result<Vec<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let users = global
			.user_loader()
			.load_many(global, list.into_iter().map(|id| id.id()))
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.map(|u| UserPartial::from_db(global, u.into()))
			.collect();

		Ok(users)
	}
}