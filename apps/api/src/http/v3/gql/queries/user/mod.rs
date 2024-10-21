use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object};
use itertools::Itertools;
use shared::database::user::{FullUser, UserId};
use shared::old_types::cosmetic::{CosmeticBadgeModel, CosmeticKind, CosmeticPaintModel};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::{UserConnectionPlatformModel, UserEditorModelPermission, UserTypeModel};
use shared::typesense::types::event::EventId;

use super::audit_log::AuditLog;
use super::emote::Emote;
use super::emote_set::EmoteSet;
use super::report::Report;
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v3::gql::guards::RateLimitGuard;
use crate::search::{search, sorted_results, SearchOptions};

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
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let editors = global
			.user_editor_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user editors"))?
			.unwrap_or_default();

		Ok(editors.into_iter().filter_map(|e| UserEditor::from_db(e, false)).collect())
	}

	async fn editor_of<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let editors = global
			.user_editor_by_editor_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user editors"))?
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

		let mut cosmetics: Vec<_> = paints
			.chain(badges)
			.map(|(kind, id)| UserCosmetic {
				id: id.into(),
				selected: self.full_user.user.style.active_paint_id.is_some_and(|p| p == id.cast())
					|| self.full_user.user.style.active_badge_id.is_some_and(|b| b == id.cast()),
				kind,
			})
			.collect();

		cosmetics.sort_by(|a, b| match a.kind.cmp(&b.kind) {
			std::cmp::Ordering::Equal => a.id.cmp(&b.id),
			other => other,
		});

		Ok(cosmetics)
	}

	async fn roles(&self) -> Result<Vec<GqlObjectId>, ApiError> {
		Ok(self.full_user.computed.roles.iter().rev().copied().map(Into::into).collect())
	}

	async fn emote_sets<'ctx>(&self, ctx: &Context<'ctx>, _entitled: Option<bool>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.unwrap_or_default();

		emote_sets.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emote_sets.into_iter().map(EmoteSet::from_db).collect())
	}

	async fn owned_emotes<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut emotes = global
			.emote_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?
			.unwrap_or_default();

		emotes.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emotes.into_iter().map(|e| Emote::from_db(global, e)).collect())
	}

	#[graphql(guard = "RateLimitGuard::search(1)")]
	async fn activity<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(maximum = 100))] limit: Option<u32>,
	) -> Result<Vec<AuditLog>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		// TODO(troy): this is likely not a very good use of our query system
		// We essentially need to know the IDs of all emote_sets owned by this user
		// so that we can find the events related to those emote_sets.
		// Ideally we should just query this on typesense using a JOIN.
		// This is a temporary solution until we have a better way to query this.
		let targets = global
			.emote_set_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.unwrap_or_default()
			.into_iter()
			.map(|s| EventId::EmoteSet(s.id))
			.chain(std::iter::once(EventId::User(self.id.id())))
			.join(", ");

		let options = SearchOptions::builder()
			.query("*".to_owned())
			.filter_by(format!("target_id: [{targets}]"))
			.sort_by(vec!["created_at:desc".to_owned()])
			.page(None)
			.per_page(limit.unwrap_or(20))
			.build();

		let result = search::<shared::typesense::types::event::Event>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let events = global
			.event_by_id_loader
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load events"))?;

		Ok(sorted_results(result.hits, events)
			.into_iter()
			.filter_map(AuditLog::from_db)
			.collect())
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
			id: if editor_of {
				value.id.user_id.into()
			} else {
				value.id.editor_id.into()
			},
			added_at: value.added_at,
			permissions: UserEditorModelPermission::from_db(&value.permissions),
			visible: true,
		})
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl UserEditor {
	async fn user<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserPartial, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load_fast(global, self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user))
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
			.active_profile_picture
			.as_ref()
			.and_then(|s| {
				s.image_set
					.outputs
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
		Ok(self.full_user.computed.roles.iter().copied().rev().map(Into::into).collect())
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
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.unwrap_or_default();

		Ok(emote_sets.into_iter().map(EmoteSet::from_db).collect())
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

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.paint_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paint"))?
			.map(|p| CosmeticPaintModel::from_db(p, &global.config.api.cdn_origin)))
	}

	async fn badge<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<CosmeticBadgeModel>, ApiError> {
		let Some(id) = self.badge_id else {
			return Ok(None);
		};

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.badge_by_id_loader
			.load(id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load badge"))?
			.map(|b| CosmeticBadgeModel::from_db(b, &global.config.api.cdn_origin)))
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
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(session.user().ok().map(|u| UserPartial::from_db(global, u.clone()).into()))
	}

	async fn user<'ctx>(&self, ctx: &Context<'ctx>, id: GqlObjectId) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let user = global
			.user_loader
			.load(global, id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user);

		Ok(user.into())
	}

	async fn user_by_connection<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		platform: UserConnectionPlatformModel,
		#[graphql(validator(max_length = 100))] id: String,
	) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let platform = shared::database::user::connection::Platform::from(platform);

		let user = match global
			.user_by_platform_id_loader
			.load((platform, id))
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
		{
			Some(u) => u,
			None => return Ok(UserPartial::deleted_user().into()),
		};

		let full_user = global
			.user_loader
			.load_fast_user(global, user)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(UserPartial::from_db(global, full_user).into())
	}

	#[graphql(guard = "RateLimitGuard::search(1)")]
	async fn users<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(max_length = 100))] query: String,
		#[graphql(validator(maximum = 10))] page: Option<u32>,
		#[graphql(validator(maximum = 100))] limit: Option<u32>,
	) -> Result<Vec<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let options = SearchOptions::builder()
			.query(query)
			.query_by(vec![
				"twitch_names".to_owned(),
				"kick_names".to_owned(),
				"google_names".to_owned(),
				"discord_names".to_owned(),
			])
			.query_by_weights(vec![4, 1, 1, 1])
			.sort_by(vec!["_text_match(buckets: 10):desc".to_owned(), "role_rank:desc".to_owned()])
			.page(page)
			.per_page(limit)
			.prioritize_exact_match(true)
			.exaustive(true)
			.build();

		let result = search::<shared::typesense::types::user::User>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let users = global
			.user_loader
			.load_fast_many(global, result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load users");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users")
			})?;

		Ok(sorted_results(result.hits, users)
			.into_iter()
			.map(|u| UserPartial::from_db(global, u))
			.collect())
	}

	#[graphql(name = "usersByID")]
	async fn users_by_id<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(max_items = 100))] list: Vec<GqlObjectId>,
	) -> Result<Vec<UserPartial>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let users = global
			.user_loader
			.load_many(global, list.into_iter().map(|id| id.id()))
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load users"))?
			.into_values()
			.map(|u| UserPartial::from_db(global, u))
			.collect();

		Ok(users)
	}
}
