use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object};
use hyper::StatusCode;
use mongodb::bson::doc;
use shared::database::user::{FullUser, UserId};
use shared::old_types::cosmetic::{CosmeticBadgeModel, CosmeticKind, CosmeticPaintModel};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{UserConnectionPlatformModel, UserTypeModel};

use super::audit_log::AuditLog;
use super::emote::Emote;
use super::emote_set::EmoteSet;
use super::report::Report;
use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::types::UserEditorModelPermission;
use crate::utils::{search, SearchOptions};

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
	full_user: FullUser,
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
			full_user: partial.full_user,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl User {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	async fn style(&self) -> Result<UserStyle, ApiError> {
		Ok(UserStyle {
			color: self.full_user.computed.highest_role_color.unwrap_or_default(),
			paint_id: self.full_user.user.style.active_paint_id.map(Into::into),
			badge_id: self.full_user.user.style.active_badge_id.map(Into::into),
		})
	}

	async fn editors<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let editors = global
			.user_editor_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, false)).collect())
	}

	async fn editor_of<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let editors = global
			.user_editor_by_editor_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, true)).collect())
	}

	async fn cosmetics(&self) -> Result<Vec<UserCosmetic>, ApiError> {
		let paints = self
			.full_user
			.computed
			.entitlements
			.paints
			.iter()
			.map(|p| (CosmeticKind::Paint, p.cast::<()>()));
		let badges = self
			.full_user
			.computed
			.entitlements
			.badges
			.iter()
			.map(|b| (CosmeticKind::Badge, b.cast::<()>()));

		let cosmetics = paints
			.chain(badges)
			.map(|(kind, id)| UserCosmetic {
				id: id.into(),
				selected: self.full_user.user.style.active_paint_id.is_some_and(|p| p == id.cast())
					|| self.full_user.user.style.active_badge_id.is_some_and(|b| b == id.cast()),
				kind,
			})
			.collect();

		Ok(cosmetics)
	}

	async fn roles(&self) -> Result<Vec<GqlObjectId>, ApiError> {
		Ok(self.full_user.computed.roles.iter().copied().map(Into::into).collect())
	}

	async fn emote_sets<'ctx>(&self, ctx: &Context<'ctx>, _entitled: Option<bool>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(emote_sets.into_iter().map(|e| EmoteSet::from_db(e)).collect())
	}

	async fn owned_emotes<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Emote>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emotes = global
			.emote_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.unwrap_or_default();

		Ok(emotes.into_iter().map(|e| Emote::from_db(global, e)).collect())
	}

	async fn activity<'ctx>(&self, _ctx: &Context<'ctx>, _limit: Option<u32>) -> Result<Vec<AuditLog>, ApiError> {
		// TODO(troy): implement
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn connections<'ctx>(&self) -> Result<Vec<UserConnection>, ApiError> {
		let emote_capacity = self
			.full_user
			.computed
			.permissions
			.emote_set_capacity
			.unwrap_or_default()
			.max(0);

		Ok(self
			.full_user
			.connections
			.iter()
			.map(|c| UserConnection::from_db(emote_capacity, c.clone(), &self.full_user.style))
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
			.user_loader
			.load_fast(global, self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u))
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
	full_user: FullUser,
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
			full_user: FullUser::default(),
		}
	}

	pub fn from_db(global: &Arc<Global>, full_user: FullUser) -> Self {
		let main_connection = full_user.connections.first();

		let avatar_url = full_user
			.style
			.active_profile_picture
			.as_ref()
			.and_then(|s| {
				s.outputs
					.iter()
					.max_by_key(|i| i.size)
					.map(|i| i.get_url(&global.config.api.cdn_origin))
			})
			.or(main_connection.and_then(|c| c.platform_avatar_url.clone()));

		Self {
			id: full_user.id.into(),
			user_type: UserTypeModel::Regular,
			username: main_connection.map(|c| c.platform_username.clone()).unwrap_or_default(),
			display_name: main_connection.map(|c| c.platform_display_name.clone()).unwrap_or_default(),
			avatar_url: avatar_url.unwrap_or_default(),
			biography: String::new(),
			full_user,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserPartial {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}

	async fn style(&self) -> Result<UserStyle, ApiError> {
		Ok(UserStyle {
			color: self.full_user.computed.highest_role_color.unwrap_or_default(),
			paint_id: self.full_user.user.style.active_paint_id.map(Into::into),
			badge_id: self.full_user.user.style.active_badge_id.map(Into::into),
		})
	}

	async fn roles(&self) -> Result<Vec<GqlObjectId>, ApiError> {
		Ok(self.full_user.computed.roles.iter().copied().map(Into::into).collect())
	}

	async fn connections(&self) -> Result<Vec<UserConnection>, ApiError> {
		let emote_capacity = self
			.full_user
			.computed
			.permissions
			.emote_set_capacity
			.unwrap_or_default()
			.max(0);

		Ok(self
			.full_user
			.connections
			.iter()
			.cloned()
			.map(|c| UserConnection::from_db(emote_capacity, c, &self.full_user.style))
			.collect())
	}

	async fn emote_sets<'ctx>(&self, ctx: &Context<'ctx>, _entitled: Option<bool>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
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
		emote_capacity: i32,
		connection: shared::database::user::connection::UserConnection,
		style: &shared::database::user::UserStyle,
	) -> Self {
		Self {
			id: connection.platform_id,
			platform: connection.platform.into(),
			username: connection.platform_username,
			display_name: connection.platform_display_name,
			linked_at: connection.linked_at,
			emote_capacity,
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
			.paint_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.and_then(|p| CosmeticPaintModel::from_db(p, &global.config.api.cdn_origin)))
	}

	async fn badge<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<CosmeticBadgeModel>, ApiError> {
		let Some(id) = self.badge_id else {
			return Ok(None);
		};

		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.badge_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.and_then(|b| CosmeticBadgeModel::from_db(b, &global.config.api.cdn_origin)))
	}
}

#[derive(Debug, Clone, Default, async_graphql::SimpleObject)]
pub struct UserSearchResult {
	pub total: u32,
	pub items: Vec<UserPartial>,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl UsersQuery {
	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<User>, ApiError> {
		let Some(session) = ctx.data_opt::<AuthSession>() else {
			return Ok(None);
		};
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let user = global
			.user_loader
			.load(global, session.user_id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u.into()));

		Ok(user.map(Into::into))
	}

	async fn user<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let user = global
			.user_loader
			.load(global, id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
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

		let user = match global
			.user_by_platform_id_loader
			.load((platform, id))
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
		{
			Some(u) => u,
			None => return Ok(UserPartial::deleted_user().into()),
		};

		let full_user = global
			.user_loader
			.load_fast_user(&global, user)
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(UserPartial::from_db(global, full_user).into())
	}

	async fn users<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		query: String,
		page: Option<u32>,
		limit: Option<u32>,
	) -> Result<Vec<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if limit.is_some_and(|l| l > 100) {
			return Err(ApiError::new_const(
				StatusCode::BAD_REQUEST,
				"limit cannot be greater than 100",
			));
		}

		if page.is_some_and(|p| p > 10) {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "page cannot be greater than 10"));
		}

		let options = SearchOptions::builder()
			.query(query)
			.query_by(vec![
				"twitch_names".to_owned(),
				"kick_names".to_owned(),
				"google_names".to_owned(),
				"discord_names".to_owned(),
			])
			.query_by_weights(vec![4, 1, 1, 1])
			.sort_by(vec!["role_rank:desc".to_owned(), "_text_match(buckets: 4):desc".to_owned()])
			.page(page)
			.per_page(limit)
			.exaustive(true)
			.build();

		let result = search::<shared::typesense::types::user::User>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let users = global
			.user_loader
			.load_fast_many(global, result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load users");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(result
			.hits
			.into_iter()
			.filter_map(|id| users.get(&id).cloned())
			.map(|u| UserPartial::from_db(global, u))
			.collect())
	}

	#[graphql(name = "usersByID")]
	async fn users_by_id<'ctx>(&self, ctx: &Context<'ctx>, list: Vec<GqlObjectId>) -> Result<Vec<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		if list.len() > 100 {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "list too large, max 100"));
		}

		let users = global
			.user_loader
			.load_many(global, list.into_iter().map(|id| id.id()))
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.into_values()
			.map(|u| UserPartial::from_db(global, u.into()))
			.collect();

		Ok(users)
	}
}
