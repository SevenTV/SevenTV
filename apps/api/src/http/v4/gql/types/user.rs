use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use shared::database::badge::BadgeId;
use shared::database::emote_set::EmoteSetId;
use shared::database::paint::PaintId;
use shared::database::role::RoleId;
use shared::database::user::profile_picture::UserProfilePictureId;
use shared::database::user::UserId;

use super::{Color, Emote, EmoteSet, Paint, Role, UserEditor, UserProfilePicture};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
	pub id: UserId,
	pub connections: Vec<UserConnection>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,

	// Computed fields
	pub highest_role_rank: i32,
	pub highest_role_color: Option<Color>,
	pub role_ids: Vec<RoleId>,

	#[graphql(skip)]
	pub(crate) full_user: shared::database::user::FullUser,
}

#[ComplexObject]
impl User {
	pub async fn main_connection(&self) -> Option<&UserConnection> {
		self.connections.first()
	}

	// TODO: Does it make sense to paginate this?
	pub async fn owned_emotes<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut emotes = global
			.emote_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?
			.unwrap_or_default();

		emotes.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emotes
			.into_iter()
			.map(|e| Emote::from_db(e, &global.config.api.cdn_origin))
			.collect())
	}

	pub async fn owned_emote_sets<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.unwrap_or_default();

		emote_sets.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emote_sets.into_iter().map(Into::into).collect())
	}

	pub async fn style<'ctx>(&self, ctx: &Context<'ctx>) -> Result<UserStyle, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(UserStyle {
			active_badge_id: self.full_user.style.active_badge_id,
			active_paint_id: self.full_user.style.active_paint_id,
			active_emote_set_id: self.full_user.style.active_emote_set_id,
			active_profile_picture_id: self.full_user.style.active_profile_picture,
			active_profile_picture: self
				.full_user
				.active_profile_picture
				.as_ref()
				.map(|p| UserProfilePicture::from_db(p.clone(), &global.config.api.cdn_origin)),
			pending_profile_picture_id: self.full_user.style.pending_profile_picture,
		})
	}

	pub async fn roles<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Role>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut loaded = global
			.role_by_id_loader
			.load_many(self.role_ids.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load roles"))?;

		let mut roles = Vec::with_capacity(loaded.len());

		for id in &self.role_ids {
			if let Some(role) = loaded.remove(id) {
				roles.push(role);
			}
		}

		Ok(roles.into_iter().map(Into::into).collect())
	}

	pub async fn editors<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let editors = global
			.user_editor_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
			.unwrap_or_default();

		Ok(editors.into_iter().map(Into::into).collect())
	}
}

impl From<shared::database::user::FullUser> for User {
	fn from(value: shared::database::user::FullUser) -> Self {
		Self {
			id: value.id,
			connections: value.connections.iter().cloned().map(Into::into).collect(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
			highest_role_rank: value.computed.highest_role_rank,
			highest_role_color: value.computed.highest_role_color.map(Color),
			role_ids: value.computed.roles.clone(),
			full_user: value,
		}
	}
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct UserStyle {
	pub active_badge_id: Option<BadgeId>,
	pub active_paint_id: Option<PaintId>,
	pub active_emote_set_id: Option<EmoteSetId>,
	pub active_profile_picture_id: Option<UserProfilePictureId>,
	pub active_profile_picture: Option<UserProfilePicture>,
	pub pending_profile_picture_id: Option<UserProfilePictureId>,
}

#[ComplexObject]
impl UserStyle {
	async fn active_paint<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<Paint>, ApiError> {
		let Some(active_paint_id) = self.active_paint_id else {
			return Ok(None);
		};

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let paint = global
			.paint_by_id_loader
			.load(active_paint_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load active paint"))?;

		Ok(paint.map(|p| Paint::from_db(p, &global.config.api.cdn_origin)))
	}

	async fn active_emote_set<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<EmoteSet>, ApiError> {
		let Some(active_emote_set_id) = self.active_emote_set_id else {
			return Ok(None);
		};

		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(active_emote_set_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load active emote set"))?;

		Ok(emote_set.map(Into::into))
	}
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UserConnection {
	pub platform: Platform,
	pub platform_id: String,
	pub platform_username: String,
	pub platform_display_name: String,
	pub platform_avatar_url: Option<String>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub linked_at: chrono::DateTime<chrono::Utc>,
	pub allow_login: bool,
}

impl From<shared::database::user::connection::UserConnection> for UserConnection {
	fn from(value: shared::database::user::connection::UserConnection) -> Self {
		Self {
			platform: value.platform.into(),
			platform_id: value.platform_id,
			platform_username: value.platform_username,
			platform_display_name: value.platform_display_name,
			platform_avatar_url: value.platform_avatar_url,
			updated_at: value.updated_at,
			linked_at: value.linked_at,
			allow_login: value.allow_login,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
pub enum Platform {
	Twitch,
	Discord,
	Google,
	Kick,
}

impl From<shared::database::user::connection::Platform> for Platform {
	fn from(value: shared::database::user::connection::Platform) -> Self {
		match value {
			shared::database::user::connection::Platform::Twitch => Self::Twitch,
			shared::database::user::connection::Platform::Discord => Self::Discord,
			shared::database::user::connection::Platform::Google => Self::Google,
			shared::database::user::connection::Platform::Kick => Self::Kick,
		}
	}
}
