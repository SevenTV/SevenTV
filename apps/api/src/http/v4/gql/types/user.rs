use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use shared::database::{
	badge::BadgeId,
	emote_set::EmoteSetId,
	paint::PaintId,
	user::{profile_picture::UserProfilePictureId, UserId},
};

use crate::{
	global::Global,
	http::error::{ApiError, ApiErrorCode},
};

use super::UserProfilePicture;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
	pub id: UserId,
	pub style: UserStyle,
	pub connections: Vec<UserConnection>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[ComplexObject]
impl User {
	pub async fn main_connection(&self) -> Option<&UserConnection> {
		self.connections.first()
	}
}

impl From<shared::database::user::User> for User {
	fn from(value: shared::database::user::User) -> Self {
		Self {
			id: value.id,
			style: value.style.into(),
			connections: value.connections.into_iter().map(Into::into).collect(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
		}
	}
}

impl From<shared::database::user::FullUser> for User {
	fn from(value: shared::database::user::FullUser) -> Self {
		Self::from(value.user)
	}
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct UserStyle {
	pub active_badge_id: Option<BadgeId>,
	pub active_paint_id: Option<PaintId>,
	pub active_emote_set_id: Option<EmoteSetId>,
	pub active_profile_picture_id: Option<UserProfilePictureId>,
	pub pending_profile_picture_id: Option<UserProfilePictureId>,
}

impl From<shared::database::user::UserStyle> for UserStyle {
	fn from(value: shared::database::user::UserStyle) -> Self {
		Self {
			active_badge_id: value.active_badge_id,
			active_paint_id: value.active_paint_id,
			active_emote_set_id: value.active_emote_set_id,
			active_profile_picture_id: value.active_profile_picture,
			pending_profile_picture_id: value.pending_profile_picture,
		}
	}
}

#[ComplexObject]
impl UserStyle {
	async fn active_profile_picture<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<UserProfilePicture>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let Some(profile_picture_id) = self.active_profile_picture_id else {
			return Ok(None);
		};

		let profile_picture = global
			.user_profile_picture_id_loader
			.load(profile_picture_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user profile picture"))?;

		Ok(profile_picture.map(|p| UserProfilePicture::from_db(p, &global.config.api.cdn_origin)))
	}

	async fn pending_profile_picture<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<UserProfilePicture>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let Some(profile_picture_id) = self.pending_profile_picture_id else {
			return Ok(None);
		};

		let profile_picture = global
			.user_profile_picture_id_loader
			.load(profile_picture_id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user profile picture"))?;

		Ok(profile_picture.map(|p| UserProfilePicture::from_db(p, &global.config.api.cdn_origin)))
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
